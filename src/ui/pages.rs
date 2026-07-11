//! Page renderers for the main content area.

use crate::app::{BmapApp, Page};
use crate::i18n::t;
use crate::model::*;
use crate::ui::empty;

use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::ActiveTheme;

/// Height of each row.
const ROW_H: f32 = 28.0;

/// Column widths.
const COL_SRC: f32 = 400.0; // file/module/section name
const COL_ARC: f32 = 200.0; // archive
const COL_FSZ: f32 = 130.0; // size
const COL_CNT: f32 = 80.0; // count
const COL_ADR: f32 = 150.0; // address
const COL_PCT: f32 = 80.0; // percentage

pub fn render(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    if app.all_entries.is_empty() {
        return div()
            .size_full()
            .child(empty::render(app, cx))
            .into_any_element();
    }

    match app.current_page {
        Page::Files | Page::Modules => {
            if let Some(ref group) = app.drilldown_group.clone() {
                symbols_view(app, cx, group.clone()).into_any_element()
            } else if app.current_page == Page::Files {
                files_view(app, cx).into_any_element()
            } else {
                modules_view(app, cx).into_any_element()
            }
        }
        Page::Sections => sections_view(app, cx).into_any_element(),
        Page::Summary => summary_view(app, cx).into_any_element(),
    }
}

// ─── Theme helpers ────────────────────────────────────────────────

fn border(cx: &App) -> Hsla {
    cx.theme().border
}
fn hover(cx: &App) -> Hsla {
    cx.theme().accent.opacity(0.1)
}
fn muted(cx: &App) -> Hsla {
    cx.theme().muted_foreground
}
fn stripe(i: usize, cx: &App) -> Hsla {
    if i % 2 == 0 {
        cx.theme().background
    } else {
        cx.theme().muted.opacity(0.3)
    }
}

// ─── Cell builders ────────────────────────────────────────────────

fn cell(w: f32, right: bool) -> Div {
    let mut d = div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .h(px(ROW_H))
        .w(px(w))
        .text_sm();
    if right {
        d = d.justify_end();
    }
    d
}

fn hdr(w: f32, right: bool, label: impl Into<SharedString>) -> Div {
    cell(w, right)
        .font_weight(FontWeight::MEDIUM)
        .child(label.into())
}

fn txt(w: f32, right: bool, s: impl Into<SharedString>) -> Div {
    cell(w, right).child(s.into())
}

fn header_row() -> Div {
    div()
        .flex()
        .flex_row()
        .border_b_1()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
}

fn page_container() -> Div {
    div().flex().flex_col().size_full().p_2().gap_1()
}

fn sort_header(app: &BmapApp, cx: &mut Context<BmapApp>, w: f32) -> impl IntoElement {
    let indicator = if app.sort_ascending { " ▲" } else { " ▼" };
    let label = format!("{}{}", t("column-size", app.language), indicator);
    hdr(w, true, label)
        .id("sort-size-header")
        .cursor_pointer()
        .on_click(cx.listener(|app, _event, _window, cx| {
            app.toggle_group_sort(cx);
        }))
}

// ─── Files view (regular children — typically < 1000 items) ──────

fn files_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let query = app.search_query.to_lowercase();
    let mut groups: Vec<&GroupSummary> = app
        .aggregate
        .module_groups
        .iter()
        .filter(|g| {
            query.is_empty()
                || g.name.to_lowercase().contains(&query)
                || source_name(&g.name).to_lowercase().contains(&query)
                || archive_name(&g.name).to_lowercase().contains(&query)
        })
        .collect();
    if app.sort_ascending {
        groups.sort_by_key(|g| g.total_size);
    }

    let b = border(cx);
    let h = hover(cx);
    let m = muted(cx);

    let header = header_row()
        .child(hdr(COL_SRC, false, t("column-source", app.language)))
        .child(hdr(COL_ARC, false, t("column-archive", app.language)))
        .child(sort_header(app, cx, COL_FSZ))
        .child(hdr(COL_CNT, true, t("symbols", app.language)));

    let rows = groups.into_iter().enumerate().map(|(i, g)| {
        let name = g.name.clone();
        div()
            .id(SharedString::from(format!("fr-{i}")))
            .flex()
            .flex_row()
            .h(px(ROW_H))
            .border_b_1()
            .border_color(b)
            .bg(stripe(i, cx))
            .hover(|s| s.bg(h))
            .cursor_pointer()
            .child(txt(COL_SRC, false, source_name(&g.name)))
            .child(txt(COL_ARC, false, archive_name(&g.name)).text_color(m))
            .child(txt(COL_FSZ, true, format_size(g.total_size)))
            .child(txt(COL_CNT, true, g.num_symbols.to_string()))
            .on_click(cx.listener(move |app, _event, _window, cx| {
                app.drill_into(name.clone(), cx);
            }))
    });

    page_container().child(header).child(
        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scrollbar()
            .children(rows),
    )
}

// ─── Modules view (regular children — typically < 1000 items) ────

fn modules_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let query = app.search_query.to_lowercase();
    let mut groups: Vec<&GroupSummary> = app
        .aggregate
        .archive_groups
        .iter()
        .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query))
        .collect();
    if app.sort_ascending {
        groups.sort_by_key(|g| g.total_size);
    }

    let b = border(cx);
    let h = hover(cx);

    let header = header_row()
        .child(hdr(COL_SRC, false, t("column-module", app.language)))
        .child(sort_header(app, cx, COL_FSZ))
        .child(hdr(COL_CNT, true, t("symbols", app.language)));

    let rows = groups.into_iter().enumerate().map(|(i, g)| {
        let name = g.name.clone();
        let display = std::path::PathBuf::from(&name)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| name.clone());
        div()
            .id(SharedString::from(format!("mr-{i}")))
            .flex()
            .flex_row()
            .h(px(ROW_H))
            .border_b_1()
            .border_color(b)
            .bg(stripe(i, cx))
            .hover(|s| s.bg(h))
            .cursor_pointer()
            .child(txt(COL_SRC, false, display))
            .child(txt(COL_FSZ, true, format_size(g.total_size)))
            .child(txt(COL_CNT, true, g.num_symbols.to_string()))
            .on_click(cx.listener(move |app, _event, _window, cx| {
                app.drill_into(name.clone(), cx);
            }))
    });

    page_container().child(header).child(
        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scrollbar()
            .children(rows),
    )
}

// ─── Sections view ────────────────────────────────────────────────

fn sections_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let categories = app.aggregate.section_categories.clone();
    let expanded = app.expanded_section.clone();
    let b = border(cx);
    let h = hover(cx);
    let m = muted(cx);

    let header = header_row()
        .child(hdr(COL_SRC, false, t("column-section", app.language)))
        .child(hdr(COL_FSZ, true, t("column-size", app.language)))
        .child(hdr(COL_CNT, true, t("symbols", app.language)));

    let mut rows: Vec<AnyElement> = Vec::new();
    for category in categories.iter() {
        let cat_name = category.name.clone();
        let is_open = expanded.as_ref() == Some(&cat_name);
        let arrow = if is_open { "▾" } else { "▸" };

        rows.push(
            div()
                .id(SharedString::from(format!("sc-{}", &cat_name)))
                .flex()
                .flex_row()
                .h(px(ROW_H))
                .border_b_1()
                .border_color(b)
                .hover(|s| s.bg(h))
                .cursor_pointer()
                .child(txt(COL_SRC, false, format!("{arrow}  {cat_name}")))
                .child(txt(COL_FSZ, true, format_size(category.total_size)))
                .child(txt(COL_CNT, true, category.num_symbols.to_string()))
                .on_click({
                    let cat_name = cat_name.clone();
                    cx.listener(move |app, _event, _window, cx| {
                        app.toggle_section(cat_name.clone(), cx);
                    })
                })
                .into_any_element(),
        );

        if is_open {
            for sg in app
                .aggregate
                .section_groups
                .iter()
                .filter(|sg| matches_category(&cat_name, &sg.name))
            {
                rows.push(
                    div()
                        .flex()
                        .flex_row()
                        .h(px(ROW_H))
                        .border_b_1()
                        .border_color(b)
                        .child(txt(COL_SRC, false, format!("    {}", sg.name)).text_color(m))
                        .child(txt(COL_FSZ, true, format_size(sg.total_size)).text_color(m))
                        .child(txt(COL_CNT, true, sg.num_symbols.to_string()).text_color(m))
                        .into_any_element(),
                );
            }
        }
    }

    page_container().child(header).child(
        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scrollbar()
            .children(rows),
    )
}

// ─── Summary view ─────────────────────────────────────────────────

fn summary_view(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let s = &app.aggregate.summary;
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
        .child(div().text_sm().text_color(muted(cx)).child(file_info))
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .child(t("summary", app.language)),
        )
        .child(div().flex().flex_col().gap_2().children(rows))
}

// ─── Symbol drill-down (uniform_list — 100k+ possible) ────────────

fn symbols_view(app: &mut BmapApp, cx: &mut Context<BmapApp>, group: String) -> impl IntoElement {
    let mut indices = drill_indices(&app.all_entries, &group, app.show_debug);

    if !app.search_query.is_empty() {
        let q = app.search_query.to_lowercase();
        indices.retain(|&i| {
            let e = &app.all_entries[i];
            e.name.to_lowercase().contains(&q)
                || e.path_str.to_lowercase().contains(&q)
                || e.section_type.to_lowercase().contains(&q)
        });
    }

    indices.sort_by_key(|&i| {
        let s = app.all_entries[i].size;
        if app.sort_ascending {
            s as i64
        } else {
            -(s as i64)
        }
    });

    let cap = if indices.len() > 5000 {
        &indices[..5000]
    } else {
        &indices
    };
    let count = cap.len();
    let total = app.aggregate.summary.total_size.max(1);

    // Extract needed fields into flat arrays for uniform_list closure
    struct Row {
        addr: u64,
        size: u64,
        name: String,
    }
    let rows: std::rc::Rc<Vec<Row>> = std::rc::Rc::new(
        cap.iter()
            .map(|&i| {
                let e = &app.all_entries[i];
                Row {
                    addr: e.address,
                    size: e.size,
                    name: e.name.clone(),
                }
            })
            .collect(),
    );

    let back_label = format!("{} ({})", t("label-back-to-files", app.language), group);
    let back_btn = div()
        .id("back-btn")
        .flex()
        .flex_row()
        .gap_1()
        .cursor_pointer()
        .text_sm()
        .text_color(cx.theme().primary)
        .child("←")
        .child(back_label)
        .on_click(cx.listener(|app, _event, _window, cx| app.drill_out(cx)));

    let header = header_row()
        .child(hdr(COL_SRC, false, t("column-name", app.language)))
        .child(hdr(COL_ADR, true, t("column-address", app.language)))
        .child(sort_header(app, cx, COL_FSZ))
        .child(hdr(COL_PCT, true, t("column-percentage", app.language)));

    page_container().child(back_btn).child(header).child(
        uniform_list("sym-ul", count, {
            let rows = rows.clone();
            move |range, _window, cx| {
                let b = border(cx);
                let h = hover(cx);
                range
                    .map(|i| {
                        let r = &rows[i];
                        let pct = (r.size as f64 / total as f64) * 100.0;
                        div()
                            .flex()
                            .flex_row()
                            .h(px(ROW_H))
                            .border_b_1()
                            .border_color(b)
                            .hover(|s| s.bg(h))
                            .child(txt(COL_SRC, false, r.name.clone()))
                            .child(txt(COL_ADR, true, format!("0x{:08X}", r.addr)))
                            .child(txt(COL_FSZ, true, format_size(r.size)))
                            .child(txt(COL_PCT, true, format!("{pct:.2}%")))
                            .into_any_element()
                    })
                    .collect()
            }
        })
        .h_full(),
    )
}
