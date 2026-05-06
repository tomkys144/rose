use crate::gui::{Message, RoseApp};
use iced::widget::{button, row, scrollable, space, text};
use iced::{Element, Length};

pub fn view_results(app: &RoseApp) -> Element<Message> {
    let header = row![
        text("Source Gene").width(Length::Fill).size(18),
        text("Target Gene").width(Length::Fill).size(18),
        text("Score").width(Length::Fill).size(18),
        text("Identity %").width(Length::Fill).size(18),
    ]
    .spacing(10);

    let results_list: Element<_> = app
        .results
        .iter()
        .fold(iced::widget::column![].spacing(10), |col, res| {
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

    let table = iced::widget::column![
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

    iced::widget::column![
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
