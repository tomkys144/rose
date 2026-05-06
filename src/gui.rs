mod processing;
mod results;
mod setup;

use crate::models::message::MessageType;
use crate::models::{message::Message as Msg, protein::MatchResult};
use iced::futures::{channel, SinkExt};
use iced::theme::Palette;
use iced::widget::pane_grid::Target;
use iced::widget::{button, column, container, opaque, row, space, stack, text};
use iced::{
    Background, Border, Color, Element, Length, Task, Theme, alignment, color, futures, stream,
    window,
};
use tokio::time;

pub fn run_app() -> iced::Result {
    iced::application(RoseApp::default, RoseApp::update, RoseApp::view)
        .theme(|_state: &RoseApp| {
            Theme::custom(
                "Forest".to_string(),
                Palette {
                    background: color!(0x002626),
                    text: color!(0xb2c2b0),
                    primary: color!(0x104110),
                    success: color!(0xc3e88d),
                    warning: color!(0xffcb6b),
                    danger: color!(0xf07178),
                },
            )
        })
        .title("ROSE - Rust Ortholog Search Engine")
        .window(window::Settings {
            maximized: true,
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

    // Methods
    pub(in crate::gui) use_api: bool,
    pub(in crate::gui) use_align: bool,

    // Output
    pub(in crate::gui) results: Vec<MatchResult>,
    pub(in crate::gui) logs: Vec<Msg>,
    pub(in crate::gui) popup_msg: Vec<Msg>,
}

#[derive(Debug, Clone)]
pub(in crate::gui) enum Message {
    // Setup Page Messages
    SourceChanged(String),
    TargetChanged(String),
    ToggleApiMethod(bool),
    ToggleAlignMethod(bool),
    StartSearch,

    // Processing Messages
    SearchCompleted(Vec<MatchResult>),
    LogReceived(Msg),

    // Results Page Messages
    ExportResults,
    BackToSetup,

    // Common Messages
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

            Message::ToggleApiMethod(use_api) => {
                self.use_api = use_api;
                Task::none()
            }

            Message::ToggleAlignMethod(use_align) => {
                self.use_align = use_align;
                Task::none()
            }

            Message::StartSearch => {
                let mut warnings = Vec::new();
                if self.source_input.is_empty() {
                    warnings.push("Source Organism");
                }
                if self.target_input.is_empty() {
                    warnings.push("Target Organism");
                }
                if !self.use_api && !self.use_align {
                    warnings.push("Mapping Method");
                }

                if !warnings.is_empty() {
                    self.popup_msg = vec![Msg {
                        msg: format!("Missing:\n{}", warnings.join("\n")),
                        msg_type: MessageType::Error,
                    }];

                    return Task::none();
                }

                self.screen = CurrentScreen::Processing;
                self.logs.clear();
                self.popup_msg.clear();

                Task::run(
                    run_engine_stream(
                        self.source_input.clone(),
                        self.target_input.clone(),
                        self.use_api,
                        self.use_align,
                    ),
                    |message| message,
                )
            }

            Message::SearchCompleted(results) => {
                self.results = results;
                self.screen = CurrentScreen::Results;
                Task::none()
            }

            Message::LogReceived(msg) => {
                self.logs.push(msg);
                Task::none()
            }

            Message::ExportResults => {
                // TODO: Implement
                println!("Not implemented yet!");
                Task::none()
            }

            Message::BackToSetup => {
                self.results.clear();
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
            .height(Length::Fill)
            .padding(40)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

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

fn overlay_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(0, 10, 10, 0.85))),
        ..Default::default()
    }
}

fn run_engine_stream(
    source: String,
    target: String,
    use_api: bool,
    use_align: bool,
) -> impl futures::Stream<Item = Message> {
    stream::channel(100, move |mut output: channel::mpsc::Sender<Message> | async move {
        let _ = output
            .send(Message::LogReceived(Msg {
                msg: format!(
                    "Initializing ROSE pipeline\n\tSource: {}\n\tTarget: {}\n\tSelected methods: {}",
                    source,
                    target,
                    "Align" //TODO
                ),
                msg_type: MessageType::Info,
            }))
            .await;

        time::sleep(std::time::Duration::from_millis(1500)).await;


        let _ = output
            .send(Message::LogReceived(Msg {
                msg: "Network latency detected. Switching to fallback mirror...".to_string(),
                msg_type: MessageType::Warning,
            }))
            .await;

        time::sleep(std::time::Duration::from_millis(800)).await;

        let _ = output
            .send(Message::LogReceived(Msg {
                msg: "Pipeline complete! Formatting results...".to_string(),
                msg_type: MessageType::Info,
            }))
            .await;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let dummy_results = vec![
            MatchResult {
                query: "HXK1".to_string(),
                target: "HXK2".to_string(),
                score: 145.0,
                identity: 85.5,
            },
            MatchResult {
                query: "GAL4".to_string(),
                target: "GAL4".to_string(),
                score: 320.0,
                identity: 99.1,
            },
            MatchResult {
                query: "CDC28".to_string(),
                target: "CDK1".to_string(),
                score: 210.0,
                identity: 76.3,
            },
        ];

        let _ = output.send(Message::SearchCompleted(dummy_results)).await;
    })
}
