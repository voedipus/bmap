//! Page renderers for the main content area.

use xilem::masonry::core::ArcStr;
use xilem::masonry::layout::AsUnit;
use xilem::style::Style as _;
use xilem::view::{
    CrossAxisAlignment, FlexExt as _, FlexSpacer, button, flex_col, flex_row, label, portal,
    sized_box, text_button, virtual_scroll,
};
use xilem::{AnyWidgetView, Color, FontWeight, WidgetView};

use crate::app::AppState;
use crate::i18n::t;
use crate::model::*;
use crate::theme::{BG, MUTED, ROW_ALT, TOOLBAR_BG};

/// Height of each row.
const ROW_H: f64 = 28.0;

/// Column widths.
const COL_SRC: f64 = 400.0; // file/module/section name
const COL_ARC: f64 = 200.0; // archive
const COL_FSZ: f64 = 130.0; // size
const COL_CNT: f64 = 80.0; // count
const COL_ADR: f64 = 150.0; // address
const COL_PCT: f64 = 80.0; // percentage

// ─── Cell and row builders ──────────────────────────────────────

/// A fixed-width, fixed-height cell. Right-aligned numbers use a flex spacer
/// so the text sits at the end of the cell.
fn cell(w: f64, right: bool, text: impl Into<ArcStr>) -> Box<AnyWidgetView<AppState>> {
    cell_colored(w, right, text, None)
}

fn cell_colored(
    w: f64,
    right: bool,
    text: impl Into<ArcStr>,
    color: Option<Color>,
) -> Box<AnyWidgetView<AppState>> {
    let text_view: Box<AnyWidgetView<AppState>> = match color {
        Some(c) => label(text).text_size(13.0).color(c).boxed(),
        None => label(text).text_size(13.0).boxed(),
    };
    let inner: Box<AnyWidgetView<AppState>> = if right {
        flex_row((FlexSpacer::Flex(1.0), text_view)).boxed()
    } else {
        flex_row((text_view, FlexSpacer::Flex(1.0))).boxed()
    };
    sized_box(inner)
        .fixed_width(w.px())
        .fixed_height(ROW_H.px())
        .boxed()
}

fn header_cell(w: f64, right: bool, text: impl Into<ArcStr>) -> Box<AnyWidgetView<AppState>> {
    let text_view = label(text).text_size(13.0).weight(FontWeight::BOLD);
    let inner: Box<AnyWidgetView<AppState>> = if right {
        flex_row((FlexSpacer::Flex(1.0), text_view)).boxed()
    } else {
        flex_row((text_view, FlexSpacer::Flex(1.0))).boxed()
    };
    sized_box(inner)
        .fixed_width(w.px())
        .fixed_height(ROW_H.px())
        .boxed()
}

/// Zebra stripe for row `i`.
fn stripe(i: usize) -> Color {
    if i.is_multiple_of(2) { BG } else { ROW_ALT }
}

/// A full-width non-interactive row. A trailing flex spacer stretches the row
/// across the available width so the background covers the whole row.
fn plain_row(cells: Vec<Box<AnyWidgetView<AppState>>>, bg: Color) -> Box<AnyWidgetView<AppState>> {
    sized_box(flex_row((cells, FlexSpacer::Flex(1.0))).background_color(bg))
        .fixed_height(ROW_H.px())
        .boxed()
}

/// A full-width clickable row (a styled button).
fn clickable_row(
    cells: Vec<Box<AnyWidgetView<AppState>>>,
    bg: Color,
    on_click: impl Fn(&mut AppState) + Send + Sync + 'static,
) -> Box<AnyWidgetView<AppState>> {
    button(flex_row((cells, FlexSpacer::Flex(1.0))), on_click)
        .background_color(bg)
        .padding(0.0.px())
        .corner_radius(0.0.px())
        .boxed()
}

/// Header row + a scrollable list of rows, all full width.
fn table_frame(
    header: Box<AnyWidgetView<AppState>>,
    rows: Vec<Box<AnyWidgetView<AppState>>>,
) -> Box<AnyWidgetView<AppState>> {
    flex_col((
        header,
        portal(flex_col(rows).cross_axis_alignment(CrossAxisAlignment::Stretch)).flex(1.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Stretch)
    .boxed()
}

fn header_bar(children: Vec<Box<AnyWidgetView<AppState>>>) -> Box<AnyWidgetView<AppState>> {
    sized_box(flex_row((children, FlexSpacer::Flex(1.0))).background_color(TOOLBAR_BG))
        .fixed_height(ROW_H.px())
        .boxed()
}

/// Clickable "Size" column header that toggles the sort direction.
fn sort_header(data: &AppState) -> Box<AnyWidgetView<AppState>> {
    let indicator = if data.sort_ascending { " ▲" } else { " ▼" };
    let label_text = format!("{}{}", t("column-size", data.language), indicator);
    let inner = flex_row((
        FlexSpacer::Flex(1.0),
        button(
            label(label_text).text_size(13.0).weight(FontWeight::BOLD),
            |data: &mut AppState| data.sort_ascending = !data.sort_ascending,
        )
        .padding(0.0.px())
        .corner_radius(0.0.px())
        .background_color(Color::TRANSPARENT),
    ));
    sized_box(inner)
        .fixed_width(COL_FSZ.px())
        .fixed_height(ROW_H.px())
        .boxed()
}

// ─── Files view ─────────────────────────────────────────────────

pub fn files_view(data: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    let query = data.search_query.to_lowercase();
    let mut groups: Vec<&GroupSummary> = data
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
    if data.sort_ascending {
        groups.sort_by_key(|g| g.total_size);
    }

    let header = header_bar(vec![
        header_cell(COL_SRC, false, t("column-source", data.language)),
        header_cell(COL_ARC, false, t("column-archive", data.language)),
        sort_header(data),
        header_cell(COL_CNT, true, t("symbols", data.language)),
    ]);

    let rows = groups
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            let name = g.name.clone();
            clickable_row(
                vec![
                    cell(COL_SRC, false, source_name(&g.name)),
                    cell_colored(COL_ARC, false, archive_name(&g.name), Some(MUTED)),
                    cell(COL_FSZ, true, format_size(g.total_size)),
                    cell(COL_CNT, true, g.num_symbols.to_string()),
                ],
                stripe(i),
                move |data: &mut AppState| data.drill_into(name.clone()),
            )
        })
        .collect();

    table_frame(header, rows)
}

// ─── Modules view ───────────────────────────────────────────────

pub fn modules_view(data: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    let query = data.search_query.to_lowercase();
    let mut groups: Vec<&GroupSummary> = data
        .aggregate
        .archive_groups
        .iter()
        .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query))
        .collect();
    if data.sort_ascending {
        groups.sort_by_key(|g| g.total_size);
    }

    let header = header_bar(vec![
        header_cell(COL_SRC, false, t("column-module", data.language)),
        sort_header(data),
        header_cell(COL_CNT, true, t("symbols", data.language)),
    ]);

    let rows = groups
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            let name = g.name.clone();
            let display = std::path::PathBuf::from(&name)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| name.clone());
            clickable_row(
                vec![
                    cell(COL_SRC, false, display),
                    cell(COL_FSZ, true, format_size(g.total_size)),
                    cell(COL_CNT, true, g.num_symbols.to_string()),
                ],
                stripe(i),
                move |data: &mut AppState| data.drill_into(name.clone()),
            )
        })
        .collect();

    table_frame(header, rows)
}

// ─── Sections view ──────────────────────────────────────────────

pub fn sections_view(data: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    let categories = data.aggregate.section_categories.clone();
    let expanded = data.expanded_section.clone();

    let header = header_bar(vec![
        header_cell(COL_SRC, false, t("column-section", data.language)),
        header_cell(COL_FSZ, true, t("column-size", data.language)),
        header_cell(COL_CNT, true, t("symbols", data.language)),
    ]);

    let mut rows: Vec<Box<AnyWidgetView<AppState>>> = Vec::new();
    let mut idx = 0usize;
    for category in categories.iter() {
        let cat_name = category.name.clone();
        let is_open = expanded.as_ref() == Some(&cat_name);
        let arrow = if is_open { "▾" } else { "▸" };

        rows.push(clickable_row(
            vec![
                cell(COL_SRC, false, format!("{arrow}  {cat_name}")),
                cell(COL_FSZ, true, format_size(category.total_size)),
                cell(COL_CNT, true, category.num_symbols.to_string()),
            ],
            stripe(idx),
            {
                let cat_name = cat_name.clone();
                move |data: &mut AppState| data.toggle_section(cat_name.clone())
            },
        ));
        idx += 1;

        if is_open {
            for sg in data
                .aggregate
                .section_groups
                .iter()
                .filter(|sg| matches_category(&cat_name, &sg.name))
            {
                rows.push(plain_row(
                    vec![
                        cell_colored(COL_SRC, false, format!("    {}", sg.name), Some(MUTED)),
                        cell_colored(COL_FSZ, true, format_size(sg.total_size), Some(MUTED)),
                        cell_colored(COL_CNT, true, sg.num_symbols.to_string(), Some(MUTED)),
                    ],
                    stripe(idx),
                ));
                idx += 1;
            }
        }
    }

    table_frame(header, rows)
}

// ─── Summary view ───────────────────────────────────────────────

pub fn summary_view(data: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    let s = &data.aggregate.summary;
    let file_info = data
        .file_path
        .as_ref()
        .map(|p| format!("File: {}", p.display()))
        .unwrap_or_default();

    let items: Vec<(ArcStr, ArcStr)> = vec![
        (
            t("total-size", data.language).into(),
            format_size(s.total_size).into(),
        ),
        (
            t("code-size", data.language).into(),
            format_size(s.text_size).into(),
        ),
        (
            t("data-size", data.language).into(),
            format_size(s.data_size + s.rodata_size).into(),
        ),
        (
            t("bss-size", data.language).into(),
            format_size(s.bss_size).into(),
        ),
        (
            t("label-other", data.language).into(),
            format_size(s.other_size).into(),
        ),
        (
            t("symbols", data.language).into(),
            s.num_symbols.to_string().into(),
        ),
        (
            t("label-object-files", data.language).into(),
            s.num_files.to_string().into(),
        ),
        (
            t("label-section-types", data.language).into(),
            s.num_sections.to_string().into(),
        ),
    ];

    let rows: Vec<Box<AnyWidgetView<AppState>>> = items
        .into_iter()
        .map(|(label_text, value)| {
            flex_row((
                sized_box(label(label_text).text_size(13.0)).fixed_width(200.0.px()),
                label(value).text_size(13.0),
            ))
            .boxed()
        })
        .collect();

    sized_box(
        flex_col((
            label(file_info).text_size(12.0).color(MUTED),
            label(t("summary", data.language))
                .text_size(20.0)
                .weight(FontWeight::BOLD),
            flex_col(rows).gap(6.0.px()),
        ))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .gap(10.0.px()),
    )
    .padding(12.0.px())
    .boxed()
}

// ─── Symbol drill-down (virtualized — 100k+ possible) ───────────

pub fn symbols_view(data: &mut AppState) -> Box<AnyWidgetView<AppState>> {
    let group = data.drilldown_group.clone().unwrap_or_default();
    let count = data.symbol_rows.len();
    let total = data.aggregate.summary.total_size.max(1);

    let back_label = format!("{} ({group})", t("label-back-to-files", data.language));
    let back_btn = text_button(back_label, |data: &mut AppState| data.drill_out())
        .padding(0.0.px())
        .background_color(Color::TRANSPARENT)
        .boxed();

    let header = header_bar(vec![
        header_cell(COL_SRC, false, t("column-name", data.language)),
        header_cell(COL_ADR, true, t("column-address", data.language)),
        sort_header(data),
        header_cell(COL_PCT, true, t("column-percentage", data.language)),
    ]);

    let list: Box<AnyWidgetView<AppState>> = if count == 0 {
        sized_box(flex_row((
            FlexSpacer::Flex(1.0),
            label(t("label-no-matches", data.language)).text_size(13.0),
            FlexSpacer::Flex(1.0),
        )))
        .padding(12.0.px())
        .boxed()
    } else {
        virtual_scroll(count, move |data: &mut AppState, i: usize| {
            let n = data.symbol_rows.len();
            let idx = data.symbol_rows[i.min(n - 1)];
            let e = &data.all_entries[idx];
            let pct = (e.size as f64 / total as f64) * 100.0;
            plain_row(
                vec![
                    cell(COL_SRC, false, e.name.clone()),
                    cell(COL_ADR, true, format!("0x{:08X}", e.address)),
                    cell(COL_FSZ, true, format_size(e.size)),
                    cell(COL_PCT, true, format!("{pct:.2}%")),
                ],
                stripe(i),
            )
        })
        .boxed()
    };

    flex_col((back_btn, header, list.flex(1.0)))
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
        .gap(6.0.px())
        .boxed()
}
