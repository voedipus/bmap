//! Modules page: groups symbols by archive/library file and supports drill-down.

use crate::app::{AppModel, Message, format_size};
use crate::fl;
use crate::views::{
    cell, divider, filter_bar, header_button, header_label, labeled_sort_header, numeric_cell,
    sort_indicator, widths,
};

use iced::widget::{button, column, row, scrollable, space};
use iced::{Alignment, Element, Length};
use std::path::PathBuf;

pub fn modules_view(model: &AppModel) -> Element<'_, Message> {
    if model.archive_groups.is_empty() {
        return crate::views::empty_view(model);
    }

    let size_label = labeled_sort_header(&fl!("column-size"), sort_indicator(model));

    let header = row![
        space::horizontal(),
        header_button(size_label, widths::SIZE, Message::ToggleGroupSort),
        header_label(fl!("symbols"), widths::SYMBOLS, Alignment::End),
    ];

    let query = model.search_query.to_lowercase();
    let filtered: Vec<_> = model
        .archive_groups
        .iter()
        .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query))
        .collect();

    let mut rows = Vec::with_capacity(filtered.len() * 2);
    for group in &filtered {
        rows.push(divider());
        let display_name = PathBuf::from(&group.name)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| group.name.clone());
        let item = button(row![
            cell(display_name, Length::Fill, Alignment::Start),
            numeric_cell(format_size(group.total_size), widths::SIZE),
            numeric_cell(group.num_symbols.to_string(), widths::SYMBOLS),
        ])
        .on_press(Message::DrillInto(group.name.clone()))
        .padding(0)
        .width(Length::Fill)
        .style(button::text);
        rows.push(item.into());
    }

    let body = scrollable(column(rows).spacing(0)).height(Length::Fill);

    column![
        filter_bar(model, &fl!("search-by-module")),
        space::horizontal().height(Length::Fixed(8.0)),
        header,
        divider(),
        body
    ]
    .spacing(0)
    .padding([0, 10])
    .height(Length::Fill)
    .into()
}
