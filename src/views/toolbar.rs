//! Top application toolbar with file open, navigation, and debug toggle.

use crate::app::{AppModel, Message, Page};
use crate::fl;
use crate::views::{nav_pill_background, toolbar_background, toolbar_border_color};

use iced::widget::{button, container, row, space, text};
use iced::{Alignment, Element, Length};

pub fn toolbar(model: &AppModel) -> Element<'_, Message> {
    let open_btn = button(text(fl!("open")).size(14))
        .on_press(Message::OpenFile)
        .padding([6, 18])
        .style(|theme, status| {
            let mut s = button::primary(theme, status);
            s.border.radius = 14.0.into();
            s
        });

    let pages = [
        (Page::Files, fl!("files")),
        (Page::Modules, fl!("modules")),
        (Page::Sections, fl!("sections")),
        (Page::Summary, fl!("summary")),
    ];

    let nav_buttons: Vec<Element<'_, Message>> = pages
        .into_iter()
        .map(|(page, label)| {
            let is_active = model.current_page == page;
            let mut btn = button(text(label).size(13));
            btn = btn.padding([5, 14]).style(move |theme, status| {
                let mut s = if is_active {
                    button::primary(theme, status)
                } else {
                    button::secondary(theme, status)
                };
                s.border.radius = 12.0.into();
                s
            });
            btn.on_press(Message::SelectPage(page)).into()
        })
        .collect();

    let nav_group = container(row(nav_buttons).spacing(2))
        .padding(2)
        .style(|theme| container::Style {
            background: Some(nav_pill_background(theme).into()),
            border: iced::Border {
                radius: 16.0.into(),
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
