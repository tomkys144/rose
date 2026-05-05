use crate::models::protein::{MatchResult, Protein};
use iced::widget::{button, checkbox, column, container, row, scrollable, space, text, text_input};
use iced::{Alignment, Element, Length, Task, Theme, window, color};
use iced::border::color;
use iced::theme::Palette;

pub fn run_app() -> iced::Result {
    iced::application(RoseApp::default, RoseApp::update, RoseApp::view)
        .theme(|_state: &RoseApp| Theme::custom(
            "Forest".to_string(),
            Palette {
                background: color!(0x002626),
                text: color!(0xb2cb0),
                primary: color!(0x104110),
                success: color!(0xc3e88d),
                warning: color!(0xffcb6b),
                danger: color!(0xf07178)
            }
        ))
        .title("ROSE - Rust Ortholog Search Engine")
        .window(window::Settings {
            maximized: true,
            ..Default::default()
        })
        .run()
}

#[derive(Debug, Clone, Default)]
enum CurrentScreen {
    #[default]
    Setup,
    Processing,
    Results,
}

#[derive(Default)]
struct RoseApp {
    screen: CurrentScreen,

    // Inputs
    source_input: String,
    target_input: String,

    // Methods
    use_api: bool,
    use_align: bool,

    // Output
    results: Vec<MatchResult>,
}

#[derive(Debug, Clone)]
enum Message {
    // Setup Page Messages
    SourceChanged(String),
    TargetChanged(String),
    ToggleApiMethod(bool),
    ToggleAlignMethod(bool),
    StartSearch,

    // Processing Messages
    SearchCompleted(Vec<MatchResult>),

    // Results Page Messages
    ExportResults,
    BackToSetup,
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
                if self.source_input.is_empty()
                    || self.target_input.is_empty()
                    || (!self.use_api && !self.use_align)
                {
                    return Task::none();
                }

                self.screen = CurrentScreen::Processing;

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

                Task::done(Message::SearchCompleted(dummy_results))
            }

            Message::SearchCompleted(results) => {
                self.results = results;
                self.screen = CurrentScreen::Results;
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
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self.screen {
            CurrentScreen::Setup => self.view_setup(),
            CurrentScreen::Processing => self.view_processing(),
            CurrentScreen::Results => self.view_results(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(40)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_setup(&self) -> Element<Message> {
        let src_col = column![
            text("Source Organism").size(20),
            text_input("e.g., Homo Erectus", &self.source_input).on_input(Message::SourceChanged),
        ]
        .spacing(10)
        .width(Length::Fill);

        let tgt_col = column![
            text("Target Organism").size(20),
            text_input("e.g., Homo Sapiens", &self.target_input).on_input(Message::TargetChanged),
        ]
        .spacing(10)
        .width(Length::Fill);

        let methods_col = column![
            text("Select Mapping Methods:").size(18),
            checkbox(self.use_api)
                .label("UniProt API (Pre-annotated)")
                .on_toggle(Message::ToggleApiMethod),
            checkbox(self.use_align)
                .label("Sequence Alignment (Smith-Waterman)")
                .on_toggle(Message::ToggleAlignMethod),
        ]
        .spacing(10)
        .width(Length::Fill);

        column![
            text("ROSE Setup").size(32),
            space::horizontal().height(20),
            row![src_col, space::vertical().width(40), tgt_col].width(Length::Fill),
            space::horizontal().height(20),
            methods_col,
            space::horizontal().height(20),
            button(text("Run").size(20))
                .on_press(Message::StartSearch)
                .padding([10, 30]),
        ]
        .spacing(20)
        .width(Length::Fill)
        .into()
    }

    fn view_processing(&self) -> Element<Message> {
        column![
            text("Running Mapping Pipeline...").size(32),
            text("Fetching genomes and calculating orthologs. This may take a moment.").size(18),
        ]
        .align_x(Alignment::Center)
        .spacing(20)
        .into()
    }

    fn view_results(&self) -> Element<Message> {
        let header = row![
            text("Source Gene").width(Length::Fill).size(18),
            text("Target Gene").width(Length::Fill).size(18),
            text("Score").width(Length::Fill).size(18),
            text("Identity %").width(Length::Fill).size(18),
        ]
        .spacing(10);

        let results_list: Element<_> = self
            .results
            .iter()
            .fold(column![].spacing(10), |col, res| {
                col.push(
                    row![
                        text(&res.query).width(Length::Fill),
                        text(&res.target).width(Length::Fill),
                        text(format!("{:.1}", res.score)).width(Length::Fill),
                        text(format!("{:.1}%", res.identity)).width(Length::Fill),
                    ]
                    .spacing(10),
                )
            })
            .into();

        let table = column![
            header,
            space::horizontal().height(10),
            scrollable(results_list).height(Length::Fill)
        ]
        .spacing(10);

        let controls = row![
            button("Back to Setup")
                .on_press(Message::BackToSetup)
                .padding([10, 20]),
            button("Export to CSV")
                .on_press(Message::ExportResults)
                .padding([10, 20]),
        ]
        .spacing(20);

        column![
            text("Mapping Results").size(32),
            table,
            space::horizontal().height(20),
            controls
        ]
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
