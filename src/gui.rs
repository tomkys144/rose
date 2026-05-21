mod processing;
mod results;
mod setup;

use crate::models::config::{AlignConfig, NcbiConfig, ParseConfig, PipelineConfig, PipelineStep};
use crate::models::context::PipelineContext;
use crate::models::message::MessageType;
use crate::models::{message::Message as Msg, protein::MatchResult};
use crate::proc;
use chrono::Local;
use directories::ProjectDirs;
use iced::Theme::Custom;
use iced::futures::{SinkExt, channel};
use iced::theme::Palette;
use iced::wgpu::naga::proc::ensure_block_returns;
use iced::widget::{button, column, container, opaque, stack, text};
use iced::{Background, Color, Element, Length, Task, Theme, alignment, window};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

const ICON: &[u8] = include_bytes!("../assets/icon.png");

const ROSE_COLORS: Palette = Palette {
    primary: Color::from_rgb8(191, 98, 54),
    background: Color::from_rgb8(227, 209, 181),
    text: Color::from_rgb8(70, 82, 100),
    success: Color::from_rgb8(60, 110, 83),
    warning: Color::from_rgb8(200, 115, 35),
    danger: Color::from_rgb8(166, 45, 32),
};

pub fn run_app() -> iced::Result {
    iced::application(RoseApp::default, RoseApp::update, RoseApp::view)
        .theme(|_state: &RoseApp| Theme::custom("ROSE theme", ROSE_COLORS))
        .title("ROSE - Rust Ortholog Search Engine")
        .window(window::Settings {
            maximized: true,
            icon: window::icon::from_file_data(ICON, Option::from(image::ImageFormat::Png)).ok(),
            ..Default::default()
        })
        .run()
}

#[derive(Debug, Clone, Default)]
pub(in crate::gui) enum CurrentScreen {
    #[default]
    Setup,
    Processing,
    Results,
}

#[derive(Default)]
pub struct RoseApp {
    pub(in crate::gui) screen: CurrentScreen,

    // Inputs
    pub(in crate::gui) source_input: String,
    pub(in crate::gui) target_input: String,

    // UI Checkbox State
    pub(in crate::gui) ui_use_api: bool,
    pub(in crate::gui) ui_use_align: bool,

    // Output
    pub(in crate::gui) ctx: PipelineContext,
    pub(in crate::gui) logs: Vec<Msg>,
    pub(in crate::gui) popup_msg: Vec<Msg>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SourceChanged(String),
    TargetChanged(String),
    ToggleApiMethod(bool),
    ToggleAlignMethod(bool),
    StartSearch,
    SearchCompleted(PipelineContext),
    LogReceived(Msg),
    ExportResults,
    ExportCompleted(Result<PathBuf, String>),
    BackToSetup,
    ClosePopup,
}

impl RoseApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SourceChanged(source) => {
                self.source_input = source;
                Task::none()
            }
            Message::TargetChanged(target) => {
                self.target_input = target;
                Task::none()
            }
            Message::ToggleApiMethod(val) => {
                self.ui_use_api = val;
                Task::none()
            }
            Message::ToggleAlignMethod(val) => {
                self.ui_use_align = val;
                Task::none()
            }

            Message::StartSearch => {
                let mut warnings = Vec::new();
                if self.source_input.trim().is_empty() {
                    warnings.push("Source Organism");
                }
                if self.target_input.trim().is_empty() {
                    warnings.push("Target Organism");
                }
                if !self.ui_use_api && !self.ui_use_align {
                    warnings.push("Mapping Method");
                }

                if !warnings.is_empty() {
                    self.popup_msg = vec![Msg {
                        msg: format!("Missing:\n{}", warnings.join("\n")),
                        msg_type: MessageType::Error,
                        branch: None,
                    }];
                    return Task::none();
                }

                self.screen = CurrentScreen::Processing;
                self.logs.clear();
                self.popup_msg.clear();

                let mut pipeline_steps = Vec::new();

                let ncbi_cfg = NcbiConfig {
                    api_key: String::new(),
                    max_retries: 3,
                };

                pipeline_steps.push(PipelineStep::FindReferenceGenome(ncbi_cfg.clone()));
                pipeline_steps.push(PipelineStep::FetchGenomeAnnotations(ncbi_cfg.clone()));

                let mut branches = Vec::new();

                if self.ui_use_api {
                    let parse_cfg = ParseConfig {
                        gap_open: -10,
                        gap_extend: -1,
                        min_identity: 30.0,
                        score_matrix: "Blosum 62".to_string(),
                    };

                    let mut parse_steps: Vec<PipelineStep> = Vec::new();

                    parse_steps.push(PipelineStep::ParseXmlAnnotations());
                    parse_steps.push(PipelineStep::FetchMissingUniprot());
                    parse_steps.push(PipelineStep::AlignFound(parse_cfg.clone()));

                    branches.push(parse_steps);
                }

                if self.ui_use_align {
                    let mut align_steps = Vec::new();

                    align_steps.push(PipelineStep::RunAlignment(AlignConfig {
                        gap_open: -10,
                        gap_extend: -1,
                        min_identity: 30.0,
                        score_matrix: "Blosum 62".to_string(),
                    }));
                    align_steps.push(PipelineStep::AlignFound(ParseConfig {
                        gap_open: -10,
                        gap_extend: -1,
                        min_identity: 30.0,
                        score_matrix: "Blosum 62".to_string(),
                    }));

                    branches.push(align_steps);
                }

                pipeline_steps.push(PipelineStep::ParallelBranches(branches));

                let final_config = PipelineConfig {
                    steps: pipeline_steps,
                };

                Task::run(
                    proc::run_pipeline(
                        self.source_input.clone(),
                        self.target_input.clone(),
                        final_config,
                    ),
                    |message| message,
                )
            }
            Message::SearchCompleted(ctx) => {
                self.ctx = ctx;
                self.screen = CurrentScreen::Results;
                Task::none()
            }
            Message::LogReceived(msg) => {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                let log_line = format!("[{}] [{:?}] {}\n", timestamp, msg.msg_type, msg.msg);

                let log_file_path = get_log_path();

                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file_path)
                {
                    let _ = file.write_all(log_line.as_bytes());
                }

                self.logs.push(msg);
                Task::none()
            }
            Message::ExportResults => {
                let results_clone = self.ctx.results.clone();
                let src_proteome_clone = self.ctx.src_proteome.clone();
                let tgt_proteome_clone = self.ctx.tgt_proteome.clone();

                Task::perform(
                    async move {
                        let file_handle = rfd::AsyncFileDialog::new()
                            .set_file_name("rose_export.csv")
                            .add_filter("CSV", &["csv"])
                            .save_file()
                            .await;

                        let file_path = match file_handle {
                            Some(handle) => handle.path().to_path_buf(),
                            None => return Err("Export canceled by user".to_string()),
                        };

                        let mut file = match std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(&file_path)
                        {
                            Ok(f) => f,
                            Err(e) => return Err(format!("Failed to create export file:\n{}", e)),
                        };

                        if let Err(e) = writeln!(
                            file,
                            "Source Gene,Source UniProt,Target Gene,Target UniProt,Score,Identity (%)"
                        ) {
                            return Err(format!("Failed to write data to file:\n{}", e));
                        }

                        for ((src_key, tgt_key), match_results) in &results_clone {
                            if let Some(res) = match_results.first() {
                                let src_prot = src_proteome_clone.get(src_key);
                                let tgt_prot = tgt_proteome_clone.get(tgt_key);

                                let src_gene =
                                    src_prot.map(|p| p.gene_id.as_str()).unwrap_or("N/A");
                                let src_uniprot = src_prot
                                    .and_then(|p| p.uniprot_id.as_deref())
                                    .unwrap_or("N/A");

                                let tgt_gene =
                                    tgt_prot.map(|p| p.gene_id.as_str()).unwrap_or("N/A");
                                let tgt_uniprot = tgt_prot
                                    .and_then(|p| p.uniprot_id.as_deref())
                                    .unwrap_or("N/A");

                                let _ = writeln!(
                                    file,
                                    "{},{},{},{},{},{:.1}",
                                    src_gene,
                                    src_uniprot,
                                    tgt_gene,
                                    tgt_uniprot,
                                    res.score,
                                    res.identity
                                );
                            }
                        }

                        Ok(file_path)
                    },
                    Message::ExportCompleted,
                )
            }

            Message::ExportCompleted(result) => {
                match result {
                    Ok(path) => {
                        self.popup_msg.push(Msg {
                            msg: format!("Successfully exported results to:\n{}", path.display()),
                            msg_type: MessageType::Info,
                            branch: None,
                        });
                    }
                    Err(err) => {
                        if err != "Export cancelled by user." {
                            self.popup_msg.push(Msg {
                                msg: err,
                                msg_type: MessageType::Error,
                                branch: None,
                            });
                        }
                    }
                }
                Task::none()
            }

            Message::BackToSetup => {
                self.ctx.clear();
                self.screen = CurrentScreen::Setup;
                Task::none()
            }
            Message::ClosePopup => {
                self.popup_msg.clear();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let base_content = match self.screen {
            CurrentScreen::Setup => setup::view(self),
            CurrentScreen::Processing => processing::view(self),
            CurrentScreen::Results => results::view_results(self),
        };

        let main_ui = container(base_content)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(popup_msg) = self.popup_msg.first() {
            let popup_txt = text(&popup_msg.msg).size(16);

            let styled_txt = match popup_msg.msg_type {
                MessageType::Info => popup_txt,
                MessageType::Warning => popup_txt.style(text::warning),
                MessageType::Error => popup_txt.style(text::danger),
            };

            let modal_card = container(
                column![
                    styled_txt,
                    container(
                        button(text("OK").size(16))
                            .on_press(Message::ClosePopup)
                            .padding([8, 20])
                    )
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Right),
                ]
                .spacing(20),
            )
            .padding(10)
            .width(Length::Shrink)
            .max_width(450)
            .style(popup_style);

            let overlay = container(opaque(modal_card))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(overlay_style);

            return stack![main_ui, overlay].into();
        }

        main_ui.into()
    }
}

fn popup_style(theme: &Theme) -> container::Style {
    let pallete = theme.palette();
    container::Style {
        background: Some(Background::Color(pallete.background)),
        ..Default::default()
    }
}

fn overlay_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(0, 10, 10, 0.85))),
        ..Default::default()
    }
}

fn get_log_path() -> PathBuf {
    let logfile = format!("rose_{}.log", Local::now().format("%Y-%m-%d"));

    if cfg!(debug_assertions) {
        PathBuf::from(logfile)
    } else {
        if let Some(proj_dirs) = ProjectDirs::from("cz", "tkysela", "ROSE") {
            let data_dir = proj_dirs.data_dir();

            let _ = fs::create_dir_all(data_dir);

            data_dir.join(logfile)
        } else {
            PathBuf::from(logfile)
        }
    }
}
