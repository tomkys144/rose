use crate::gui::{Message, RoseApp};
use crate::models::message::MessageType;
use iced::widget::{button, container, row, scrollable, space, text};
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
        .ctx
        .results
        .iter()
        .filter(|(_, res)| !res.is_empty())
        .fold(iced::widget::column![].spacing(10), |col, (key, res)| {
            let (score_str, identity_str) = if let Some(first_match) = res.first() {
                (
                    format!("{:.1}", first_match.score),
                    format!("{:.1}%", first_match.identity),
                )
            } else {
                ("N/A".to_string(), "N/A".to_string())
            };

            let src = &app.ctx.src_proteome[key.0.as_str()];
            let src_id = if let Some(up_id) = &src.uniprot_id {
                format!("uniprot|{}", up_id)
            } else {
                format!("EMBL|{}", src.gene_id)
            };

            let tgt = &app.ctx.tgt_proteome[key.1.as_str()];
            let tgt_id = if let Some(up_id) = &tgt.uniprot_id {
                format!("uniprot|{}", up_id)
            } else {
                format!("EMBL|{}", tgt.gene_id)
            };

            col.push(
                row![
                    text(src_id).width(Length::Fill),
                    text(tgt_id).width(Length::Fill),
                    text(score_str).width(Length::Fill),
                    text(identity_str).width(Length::Fill),
                ]
                .spacing(10),
            )
        })
        .into();

    let table = iced::widget::column![
        header,
        space::vertical().height(10),
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

    container(
        iced::widget::column![
            text("Mapping Results").size(32),
            table,
            space::vertical().height(20),
            controls
        ]
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .into()
}
