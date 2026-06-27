//! Empty state shown before a file is loaded or when a view has no matches.

use crate::app::{AppModel, Message};
use crate::fl;
use crate::views::icons;

use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length};

pub fn empty_view(model: &AppModel) -> Element<'_, Message> {
    let msg: Element<'_, Message> = if let Some(err) = &model.error {
        column![
            text(icons::WARNING).size(48),
            text(fl!("error-loading")).size(24),
            text(err.clone()).size(14),
        ]
        .spacing(10)
        .align_x(Alignment::Center)
        .into()
    } else {
        column![
            text(icons::FILES).size(64),
            text(fl!("no-file-loaded")).size(24),
            text(fl!("open-instruction")).size(15),
        ]
        .spacing(10)
        .align_x(Alignment::Center)
        .into()
    };

    container(msg)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(32)
        .into()
}
