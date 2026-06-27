//! Page renderers for the main content area.

use crate::app::{BmapApp, Page};
use crate::i18n::t;
use crate::model::*;
use crate::theme;
use crate::ui::empty;
use crate::ui::table;
use gpui::prelude::*;
use gpui::*;

pub fn render(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    if app.all_entries.is_empty() {
        return div().size_full().child(empty::render(app));
    }

    let content = match app.current_page {
        Page::Files | Page::Modules => group_or_symbols(app, cx),
        Page::Sections => sections_view(app, cx),
        Page::Summary => summary_view(app, cx),
    };
    div().size_full().child(content)
}

fn page_container() -> Div {
    div().flex().flex_col().size_full().p_2().gap_2()
}

fn group_or_symbols(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> Div {
    if let Some(ref group) = app.drilldown_group {
        symbols_view(app, cx, group.clone())
    } else if app.current_page == Page::Files {
        files_view(app, cx)
    } else {
        modules_view(app, cx)
    }
}

fn files_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> Div {
    let query = app.search_query.to_string().to_lowercase();
    let filtered: Vec<GroupSummary> = app
        .module_groups
        .iter()
        .filter(|g| {
            query.is_empty()
                || g.name.to_lowercase().contains(&query)
                || source_name(&g.name).to_lowercase().contains(&query)
                || archive_name(&g.name).to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    let header = table::header_row()
        .child(table::header_row_cell(
            t("column-source", app.language),
            None,
        ))
        .child(table::header_row_cell(
            t("column-archive", app.language),
            Some(px(200.0)),
        ))
        .child(sortable_size_header(app, cx, px(130.0), true))
        .child(table::header_row_cell(
            t("symbols", app.language),
            Some(px(80.0)),
        ));

    let rows = filtered.into_iter().enumerate().map(|(i, group)| {
        let group_name = SharedString::from(group.name.clone());
        table::row()
            .id(format!("file-row-{i}"))
            .cursor_pointer()
            .child(table::text_cell(
                SharedString::from(source_name(&group.name)),
                None,
            ))
            .child(table::text_cell(
                SharedString::from(archive_name(&group.name)),
                Some(px(200.0)),
            ))
            .child(table::numeric_cell(
                format_size(group.total_size),
                px(130.0),
            ))
            .child(table::numeric_cell(group.num_symbols.to_string(), px(80.0)))
            .on_click(cx.listener(move |app, _event, _window, cx| {
                app.drill_into(group_name.clone(), cx);
            }))
    });

    page_container().child(header).child(
        div()
            .id("files-list")
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .children(rows),
    )
}

fn modules_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> Div {
    let query = app.search_query.to_string().to_lowercase();
    let filtered: Vec<GroupSummary> = app
        .archive_groups
        .iter()
        .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query))
        .cloned()
        .collect();

    let header = table::header_row()
        .child(table::header_row_cell(
            t("column-module", app.language),
            None,
        ))
        .child(sortable_size_header(app, cx, px(130.0), true))
        .child(table::header_row_cell(
            t("symbols", app.language),
            Some(px(80.0)),
        ));

    let rows = filtered.into_iter().enumerate().map(|(i, group)| {
        let group_name = SharedString::from(group.name.clone());
        let display = SharedString::from(
            std::path::PathBuf::from(&group.name)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| group.name.clone()),
        );
        table::row()
            .id(format!("module-row-{i}"))
            .cursor_pointer()
            .child(table::text_cell(display, None))
            .child(table::numeric_cell(
                format_size(group.total_size),
                px(130.0),
            ))
            .child(table::numeric_cell(group.num_symbols.to_string(), px(80.0)))
            .on_click(cx.listener(move |app, _event, _window, cx| {
                app.drill_into(group_name.clone(), cx);
            }))
    });

    page_container().child(header).child(
        div()
            .id("modules-list")
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .children(rows),
    )
}

fn sections_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> Div {
    let categories = app.section_categories.clone();

    let header = table::header_row()
        .child(table::header_row_cell(
            t("column-section", app.language),
            None,
        ))
        .child(table::header_row_cell(
            t("column-size", app.language),
            Some(px(130.0)),
        ))
        .child(table::header_row_cell(
            t("symbols", app.language),
            Some(px(80.0)),
        ));

    let items = categories.into_iter().enumerate().map(|(i, category)| {
        let name = SharedString::from(category.name.clone());
        let is_open =
            app.expanded_section.as_ref() == Some(&SharedString::from(category.name.clone()));
        let arrow = if is_open { "▼" } else { "▶" };

        let header_row = table::row()
            .id(format!("section-cat-{i}"))
            .cursor_pointer()
            .child(table::text_cell(
                format!("{}  {}", arrow, category.name),
                None,
            ))
            .child(table::numeric_cell(
                format_size(category.total_size),
                px(130.0),
            ))
            .child(table::numeric_cell(
                category.num_symbols.to_string(),
                px(80.0),
            ))
            .on_click(cx.listener(move |app, _event, _window, cx| {
                app.toggle_section(name.clone(), cx);
            }));

        let mut item = div().flex().flex_col().child(header_row);
        if is_open {
            let sub_rows = app
                .section_groups
                .iter()
                .filter(|sg| matches_category(&category.name, &sg.name))
                .map(|sg| {
                    table::row()
                        .child(table::text_cell(format!("    {}", sg.name), None))
                        .child(table::numeric_cell(format_size(sg.total_size), px(130.0)))
                        .child(table::numeric_cell(sg.num_symbols.to_string(), px(80.0)))
                });
            item = item.child(div().flex().flex_col().children(sub_rows));
        }
        item
    });

    page_container().child(header).child(
        div()
            .id("sections-list")
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .children(items),
    )
}

fn summary_view(app: &mut BmapApp, _cx: &mut Context<BmapApp>) -> Div {
    let s = &app.summary;
    let file_info = app
        .file_path
        .as_ref()
        .map(|p| format!("File: {}", p.display()))
        .unwrap_or_default();

    let items: Vec<(SharedString, SharedString)> = vec![
        (
            t("total-size", app.language),
            format_size(s.total_size).into(),
        ),
        (
            t("code-size", app.language),
            format_size(s.text_size).into(),
        ),
        (
            t("data-size", app.language),
            format_size(s.data_size + s.rodata_size).into(),
        ),
        (t("bss-size", app.language), format_size(s.bss_size).into()),
        (
            t("label-other", app.language),
            format_size(s.other_size).into(),
        ),
        (t("symbols", app.language), s.num_symbols.to_string().into()),
        (
            t("label-object-files", app.language),
            s.num_files.to_string().into(),
        ),
        (
            t("label-section-types", app.language),
            s.num_sections.to_string().into(),
        ),
    ];

    let rows = items.into_iter().map(|(label, value)| {
        div()
            .flex()
            .flex_row()
            .gap_4()
            .child(div().w_48().child(label))
            .child(div().text_sm().child(value))
    });

    page_container()
        .child(
            div()
                .text_sm()
                .text_color(theme::TEXT_SECONDARY)
                .child(file_info),
        )
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .child(t("summary", app.language)),
        )
        .child(div().flex().flex_col().gap_2().children(rows))
}

fn symbols_view(app: &mut BmapApp, cx: &mut Context<BmapApp>, group: SharedString) -> Div {
    let base = app.visible_entries();
    let by_group = filter_by_group_path(&base, &group);
    let mut filtered = filter_entries(&by_group, &app.search_query);
    sort_entries(&mut filtered, app.sort_ascending);

    let back_label = format!("{} ({})", t("label-back-to-files", app.language), group);
    let back_btn = div()
        .id("back-btn")
        .flex()
        .flex_row()
        .gap_1()
        .cursor_pointer()
        .text_sm()
        .text_color(theme::ACCENT)
        .child("←")
        .child(back_label)
        .on_click(cx.listener(|app, _event, _window, cx| app.drill_out(cx)));

    let header = table::header_row()
        .child(table::header_row_cell(t("column-name", app.language), None))
        .child(table::header_row_cell(
            t("column-address", app.language),
            Some(px(150.0)),
        ))
        .child(sortable_size_header(app, cx, px(110.0), false))
        .child(table::header_row_cell(
            t("column-percentage", app.language),
            Some(px(80.0)),
        ));

    let total = app.summary.total_size.max(1);
    let rows = filtered.iter().take(5000).enumerate().map(|(i, entry)| {
        let pct = (entry.size as f64 / total as f64) * 100.0;
        table::row()
            .id(format!("symbol-row-{i}"))
            .child(table::text_cell(
                SharedString::from(entry.name.clone()),
                None,
            ))
            .child(table::numeric_cell(
                format!("0x{:08X}", entry.address),
                px(150.0),
            ))
            .child(table::numeric_cell(format_size(entry.size), px(110.0)))
            .child(table::numeric_cell(format!("{pct:.2}%"), px(80.0)))
    });

    page_container().child(back_btn).child(header).child(
        div()
            .id("symbols-list")
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .children(rows),
    )
}

fn sortable_size_header(
    app: &BmapApp,
    cx: &mut Context<BmapApp>,
    width: Pixels,
    is_group: bool,
) -> impl IntoElement {
    let indicator = if app.sort_ascending { " ▲" } else { " ▼" };
    let label = format!("{}{}", t("column-size", app.language), indicator);
    table::header_row_cell(label, Some(width))
        .id("sort-size")
        .cursor_pointer()
        .on_click(cx.listener(move |app, _event, _window, cx| {
            if is_group {
                app.toggle_group_sort(cx);
            } else {
                app.sort_ascending = !app.sort_ascending;
                cx.notify();
            }
        }))
}
