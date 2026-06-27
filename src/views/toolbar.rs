//! Top application toolbar with file open, navigation, and debug toggle.

use crate::app::{AppModel, Message, Page};
use crate::fl;
use crate::views::icons;
use crate::views::{nav_pill_background, toolbar_background, toolbar_border_color};

use iced::widget::{button, container, row, space, text};
use iced::{Alignment, Element, Length};

pub fn toolbar(model: &AppModel) -> Element<'_, Message> {
    let open_btn = button(
        row![text(icons::OPEN).size(18), text(fl!("open")).size(15),]
            .align_y(Alignment::Center)
            .spacing(7),
    )
    .on_press(Message::OpenFile)
    .padding([8, 20]);

    let pages = [
        (Page::Files, icons::FILES, fl!("files")),
        (Page::Modules, icons::MODULES, fl!("modules")),
        (Page::Sections, icons::SECTIONS, fl!("sections")),
        (Page::Summary, icons::SUMMARY, fl!("summary")),
    ];

    let nav_buttons: Vec<Element<'_, Message>> = pages
        .into_iter()
        .map(|(page, icon, label)| {
            let is_active = model.current_page == page;
            let mut btn = button(
                row![text(icon).size(14), text(label).size(13)]
                    .align_y(Alignment::Center)
                    .spacing(4)
                    .padding([4, 10]),
            );
            if !is_active {
                btn = btn.style(button::secondary);
            }
            btn.on_press(Message::SelectPage(page)).into()
        })
        .collect();

    let nav_group = container(row(nav_buttons).spacing(2))
        .padding(2)
        .style(|theme| container::Style {
            background: Some(nav_pill_background(theme).into()),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..container::Style::default()
        });

    container(
        row![
            open_btn,
            space::horizontal().width(Length::Fixed(16.0)),
            nav_group,
        ]
        .align_y(Alignment::Center)
        .spacing(12)
        .padding([8, 14]),
    )
    .style(|theme| container::Style {
        background: Some(toolbar_background(theme).into()),
        border: iced::Border {
            color: toolbar_border_color(theme),
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .width(Length::Fill)
    .into()
}
