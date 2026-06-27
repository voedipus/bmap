//! Shared view helpers and re-exports for all application pages.

pub mod empty;
pub mod files;
pub mod modules;
pub mod sections;
pub mod summary;
pub mod symbols;
pub mod toolbar;

pub use empty::empty_view;
pub use files::source_files_view;
pub use modules::modules_view;
pub use sections::sections_view;
pub use summary::summary_view;
pub use symbols::symbols_view;
pub use toolbar::toolbar;

use crate::app::{AppModel, Message};
use crate::fl;
pub use crate::model::SortColumn;
use iced::widget::{button, checkbox, container, row, space, text, text_input};
use iced::{Alignment, Element, Length, Theme};

/// Icons used in the toolbar and empty states.
pub mod icons {
    pub const OPEN: &str = "\u{1F4C2}";
    pub const FILES: &str = "\u{1f4e6}";
    pub const MODULES: &str = "\u{1f4c1}";
    pub const SECTIONS: &str = "\u{2195}";
    pub const SUMMARY: &str = "\u{2261}";
    pub const BACK: &str = "\u{2190}";
    pub const EXPANDED: &str = "\u{25BC}";
    pub const COLLAPSED: &str = "\u{25B6}";
    pub const SORT_ASC: &str = "\u{25B2}";
    pub const SORT_DESC: &str = "\u{25BC}";
    pub const WARNING: &str = "\u{26A0}";
}

/// Returns the appropriate sort indicator arrow for the current sort direction.
pub fn sort_indicator(model: &AppModel) -> &'static str {
    if model.sort_ascending {
        icons::SORT_ASC
    } else {
        icons::SORT_DESC
    }
}

/// Joins a column label with a sort indicator arrow.
pub fn labeled_sort_header(base: &str, indicator: &str) -> String {
    format!("{base}{indicator}")
}

/// A clickable column header that emits a message when pressed.
pub fn header_button(label: String, width: Length, on_press: Message) -> Element<'static, Message> {
    button(
        text(label)
            .size(13)
            .align_x(Alignment::End)
            .width(Length::Fill),
    )
    .width(width)
    .on_press(on_press)
    .padding([4, 10])
    .style(button::text)
    .into()
}

/// A static, non-clickable column header.
pub fn header_label(label: String, width: Length, align: Alignment) -> Element<'static, Message> {
    container(text(label).size(13).align_x(align).width(Length::Fill))
        .padding([4, 10])
        .width(width)
        .into()
}

/// A generic table cell with the given alignment.
pub fn cell(label: String, width: Length, align: Alignment) -> Element<'static, Message> {
    container(text(label).size(14).align_x(align).width(Length::Fill))
        .padding([5, 12])
        .width(width)
        .into()
}

/// A table cell aligned to the end, suitable for numbers.
pub fn numeric_cell(label: String, width: Length) -> Element<'static, Message> {
    cell(label, width, Alignment::End)
}

/// A horizontal divider line that adapts to the current theme.
/// Search filter bar with a text input and debug-symbol toggle.
pub fn filter_bar<'a>(model: &'a AppModel, placeholder: &str) -> Element<'a, Message> {
    let search = text_input(placeholder, &model.search_query)
        .on_input(Message::SearchChanged)
        .padding([7, 12])
        .width(Length::Fill);

    let debug = row![
        checkbox(model.show_debug)
            .on_toggle(Message::ToggleDebug)
            .size(13),
        text(fl!("label-debug")).size(12),
    ]
    .align_y(Alignment::Center)
    .spacing(4);

    row![search, debug]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
}

pub fn divider() -> Element<'static, Message> {
    container(space::horizontal())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|theme| container::Style {
            background: Some(divider_color(theme).into()),
            ..Default::default()
        })
        .into()
}

/// Color used for the toolbar/nav background in the current theme.
pub fn toolbar_background(theme: &Theme) -> iced::Color {
    match theme {
        Theme::Light => iced::Color::from_rgba(0.92, 0.92, 0.96, 1.0),
        _ => iced::Color::from_rgba(0.10, 0.10, 0.14, 1.0),
    }
}

/// Color used for the toolbar bottom border in the current theme.
pub fn toolbar_border_color(theme: &Theme) -> iced::Color {
    match theme {
        Theme::Light => iced::Color::from_rgba(0.78, 0.78, 0.84, 1.0),
        _ => iced::Color::from_rgba(0.16, 0.16, 0.22, 1.0),
    }
}

/// Color used for the grouped nav pill background in the current theme.
pub fn nav_pill_background(theme: &Theme) -> iced::Color {
    match theme {
        Theme::Light => iced::Color::from_rgba(0.84, 0.84, 0.88, 1.0),
        _ => iced::Color::from_rgba(0.12, 0.12, 0.16, 1.0),
    }
}

/// Color used for horizontal divider lines in the current theme.
pub fn divider_color(theme: &Theme) -> iced::Color {
    match theme {
        Theme::Light => iced::Color::from_rgba(0.80, 0.80, 0.86, 1.0),
        _ => iced::Color::from_rgba(0.20, 0.20, 0.26, 1.0),
    }
}

/// Column width constants used across table views.
pub mod widths {
    use iced::Length;

    pub const SIZE: Length = Length::Fixed(130.0);
    pub const SIZE_WIDE: Length = Length::Fixed(150.0);
    pub const SYMBOLS: Length = Length::Fixed(80.0);
    pub const MODULE: Length = Length::Fixed(200.0);
    pub const ADDRESS: Length = Length::Fixed(150.0);
    pub const PERCENTAGE: Length = Length::Fixed(80.0);
}
