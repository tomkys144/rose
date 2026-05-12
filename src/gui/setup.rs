use iced::widget::{button, checkbox, column, row, space, text, text_input};
use iced::{Element, Length};

use crate::gui::{Message, RoseApp};

pub fn view(app: &RoseApp) -> Element<Message> {
    let src_col = column![
        text("Source Organism").size(20),
        text_input("e.g., Homo Erectus", &app.source_input).on_input(Message::SourceChanged),
    ].spacing(10).width(Length::Fill);

    let tgt_col = column![
        text("Target Organism").size(20),
        text_input("e.g., Homo Sapiens", &app.target_input).on_input(Message::TargetChanged),
    ].spacing(10).width(Length::Fill);

    let methods_col = iced::widget::column![
        text("Select Mapping Methods:").size(18),
        checkbox(app.ui_use_api).label("UniProt API (Pre-annotated)")
            .on_toggle(Message::ToggleApiMethod),
        checkbox(app.ui_use_align).label("Sequence Alignment (Smith-Waterman)")
            .on_toggle(Message::ToggleAlignMethod),
    ].spacing(10).width(Length::Fill);

    iced::widget::column![
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