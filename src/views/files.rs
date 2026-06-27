//! Source files page: groups symbols by object file and supports drill-down.

use crate::app::{AppModel, Message};
use crate::fl;
use crate::model::GroupSummary;
use crate::views::{
    cell, divider, filter_bar, header_button, header_label, labeled_sort_header, numeric_cell,
    sort_indicator, widths,
};

use iced::widget::{button, column, row, scrollable, space};
use iced::{Alignment, Element, Length};
use std::path::PathBuf;

pub fn source_files_view(model: &AppModel) -> Element<'_, Message> {
    if model.module_groups.is_empty() {
        return crate::views::empty_view(model);
    }

    let size_label = labeled_sort_header(&fl!("column-size"), sort_indicator(model));

    let header = row![
        space::horizontal(),
        header_label(fl!("column-module"), widths::MODULE, Alignment::Start),
        header_button(size_label, widths::SIZE, Message::ToggleGroupSort),
        header_label(fl!("symbols"), widths::SYMBOLS, Alignment::End),
    ];

    let query = model.search_query.to_lowercase();
    let filtered: Vec<&GroupSummary> = model
        .module_groups
        .iter()
        .filter(|g| {
            query.is_empty()
                || g.name.to_lowercase().contains(&query)
                || source_name(&g.name).to_lowercase().contains(&query)
                || archive_name(&g.name).to_lowercase().contains(&query)
        })
        .collect();

    let mut rows = Vec::with_capacity(filtered.len() * 2);
    for group in &filtered {
        rows.push(divider());
        let src = source_name(&group.name);
        let arc = archive_name(&group.name);
        let item = button(row![
            cell(src, Length::Fill, Alignment::Start),
            cell(arc, widths::MODULE, Alignment::Start),
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
        filter_bar(model, &fl!("search-by-source")),
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

/// Extracts the source object name from an archive path like `libfoo.a(foo.o)`.
fn source_name(object_path: &str) -> String {
    let inner = if let Some(start) = object_path.find('(') {
        if let Some(end) = object_path.find(')') {
            &object_path[start + 1..end]
        } else {
            &object_path[start + 1..]
        }
    } else {
        object_path
    };
    PathBuf::from(inner)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Extracts the archive or file name from an object path.
fn archive_name(object_path: &str) -> String {
    if let Some(p) = object_path.find('(') {
        let archive = &object_path[..p];
        PathBuf::from(archive)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| archive.to_string())
    } else {
        PathBuf::from(object_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| object_path.to_string())
    }
}

// Re-exported here because it is only used by view code.
pub use crate::app::format_size;
