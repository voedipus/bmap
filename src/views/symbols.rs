//! Symbol drill-down page: lists individual symbols for a selected group.

use crate::app::{AppModel, Message, format_size};
use crate::fl;
use crate::model::{filter_entries, sort_entries};
use crate::views::SortColumn;
use crate::views::{
    cell, divider, filter_bar, header_button, header_label, icons, labeled_sort_header,
    numeric_cell, sort_indicator, widths,
};

use iced::widget::{button, column, row, scrollable, space, text};
use iced::{Alignment, Element, Length};
use std::path::PathBuf;

pub fn symbols_view(model: &AppModel) -> Element<'_, Message> {
    let base = model.visible_entries();
    let by_group = if let Some(ref group) = model.drilldown_group {
        filter_by_group(&base, group)
    } else {
        base
    };

    let mut filtered = filter_entries(&by_group, &model.search_query);
    sort_entries(&mut filtered, SortColumn::Size, model.sort_ascending);

    if filtered.is_empty() && model.all_entries.is_empty() {
        return crate::views::empty_view(model);
    }

    let mut content = column![].spacing(0);

    if let Some(ref group) = model.drilldown_group {
        let short_name = PathBuf::from(group)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| group.clone());
        content = content.push(
            button(
                row![
                    text(icons::BACK).size(13),
                    text(format!("{} ({})", fl!("label-back-to-files"), short_name)).size(14)
                ]
                .align_y(Alignment::Center)
                .spacing(5),
            )
            .on_press(Message::DrillOut)
            .style(button::text)
            .padding([4, 0]),
        );
        content = content.push(space::horizontal().height(Length::Fixed(6.0)));
    }

    content = content.push(filter_bar(model, &fl!("search-placeholder")));
    content = content.push(space::horizontal().height(Length::Fixed(8.0)));

    let size_label = labeled_sort_header(&fl!("column-size"), sort_indicator(model));

    let header = row![
        space::horizontal(),
        header_label(fl!("column-address"), widths::ADDRESS, Alignment::End),
        header_button(
            size_label,
            widths::SIZE_WIDE,
            Message::SortBy(SortColumn::Size)
        ),
        header_label(fl!("column-percentage"), widths::PERCENTAGE, Alignment::End),
    ];
    content = content.push(header);

    let total = model.summary.total_size.max(1);
    // Cap the rendered rows to keep scrolling responsive on huge binaries.
    let row_count = filtered.len().min(5000);
    let mut rows = Vec::with_capacity(row_count);
    for entry in filtered.iter().take(row_count) {
        let pct = if total > 0 {
            (entry.size as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        rows.push(
            row![
                cell(entry.name.clone(), Length::Fill, Alignment::Start),
                cell(
                    format!("0x{:08X}", entry.address),
                    widths::ADDRESS,
                    Alignment::End,
                ),
                numeric_cell(format_size(entry.size), widths::SIZE_WIDE),
                numeric_cell(format!("{pct:.2}%"), widths::PERCENTAGE),
            ]
            .into(),
        );
    }

    let body: Element<'_, Message> = if rows.is_empty() {
        text(fl!("label-no-matches")).into()
    } else {
        scrollable(column(rows).spacing(1))
            .height(Length::Fill)
            .into()
    };
    content = content.push(divider());
    content = content.push(body);

    content.padding([0, 10]).height(Length::Fill).into()
}

/// Filters entries whose file path matches the selected group.
///
/// A group may be either an exact object file path or the outer archive path
/// of an entry like `libfoo.a(foo.o)`.
fn filter_by_group(
    entries: &[crate::model::MapEntry],
    group_path: &str,
) -> Vec<crate::model::MapEntry> {
    entries
        .iter()
        .filter(|e| {
            let entry_path = e.filepath.to_string_lossy();
            if entry_path == group_path {
                return true;
            }
            if let Some(p) = entry_path.find('(') {
                return &entry_path[..p] == group_path;
            }
            false
        })
        .cloned()
        .collect()
}
