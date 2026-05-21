use crate::gui::{Message, RoseApp};
use crate::models::message::MessageType;
use chrono::{DateTime, Local};
use iced::widget::{column, container, row, scrollable, space, text};
use iced::{Alignment, Element, Length, theme};
use std::time::SystemTime;

pub fn view(app: &RoseApp) -> Element<Message> {
    let log_list = app.logs.iter().fold(column![].spacing(5), |col, log| {
        let tm = SystemTime::now();
        let dtm: DateTime<Local> = tm.into();
        let dtm_str = dtm.format("%Y-%m-%d %H:%M:%S").to_string();
        let tm_stamp = text(dtm_str);
        let styled_tm_stamp = match log.msg_type {
            MessageType::Info => tm_stamp,
            MessageType::Warning => tm_stamp.style(text::warning),
            MessageType::Error => tm_stamp.style(text::danger),
        };

        let log_txt = text(&log.msg);

        let styled_txt = match log.msg_type {
            MessageType::Info => log_txt,
            MessageType::Warning => log_txt.style(text::warning),
            MessageType::Error => log_txt.style(text::danger),
        };

        let entry = if let Some(branch) = &log.branch {
            row![
                styled_tm_stamp,
                text(format!("[{}] ", branch)).style(text::secondary),
                styled_txt
            ]
            .spacing(20)
            .align_y(Alignment::Start)
        } else {
            row![
                styled_tm_stamp,
                text(format!("[{}] ", "Main")).style(text::secondary),
                styled_txt
            ]
            .spacing(20)
            .align_y(Alignment::Start)
        };

        col.push(entry)
    });

    let log_term = container(scrollable(log_list))
        .width(Length::FillPortion(80))
        .height(Length::FillPortion(80))
        .padding(15);

    container(
        column![
            text("Running ROSE pipeline...").size(20),
            space::vertical().height(20),
            log_term
        ]
        .align_x(Alignment::Center)
        .spacing(20),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
