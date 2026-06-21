use crate::app::{AppModel, Message, Page, format_size};
use crate::fl;
use crate::model::*;

use iced::widget::{button, checkbox, column, container, row, scrollable, space, text, text_input};
use iced::{Alignment, Element, Length, Theme};
use std::path::PathBuf;

pub fn toolbar(model: &AppModel) -> Element<'_, Message> {
    let open_btn = button(
        row![text("\u{1F4C2}").size(18), text(fl!("open")).size(15),]
            .align_y(Alignment::Center)
            .spacing(7),
    )
    .on_press(Message::OpenFile)
    .padding([8, 20]);

    let pages = [
        (Page::Files, "\u{1f4e6}", fl!("files")),
        (Page::Modules, "\u{1f4c1}", fl!("modules")),
        (Page::Sections, "\u{2195}", fl!("sections")),
        (Page::Summary, "\u{2261}", fl!("summary")),
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
            background: Some(
                match theme {
                    Theme::Light => iced::Color::from_rgba(0.84, 0.84, 0.88, 1.0),
                    _ => iced::Color::from_rgba(0.12, 0.12, 0.16, 1.0),
                }
                .into(),
            ),
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
        background: Some(
            match theme {
                Theme::Light => iced::Color::from_rgba(0.92, 0.92, 0.96, 1.0),
                _ => iced::Color::from_rgba(0.10, 0.10, 0.14, 1.0),
            }
            .into(),
        ),
        border: iced::Border {
            color: match theme {
                Theme::Light => iced::Color::from_rgba(0.78, 0.78, 0.84, 1.0),
                _ => iced::Color::from_rgba(0.16, 0.16, 0.22, 1.0),
            },
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .width(Length::Fill)
    .into()
}

fn sort_indicator(model: &AppModel) -> &'static str {
    if model.sort_ascending {
        " \u{25B2}"
    } else {
        " \u{25BC}"
    }
}

fn mk_label(base: &str, indicator: &str) -> String {
    format!("{base}{indicator}")
}

fn hdr_btn(label: String, width: Length, on_press: Message) -> Element<'static, Message> {
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

fn hdr_lbl(label: String, width: Length, align: Alignment) -> Element<'static, Message> {
    container(text(label).size(13).align_x(align).width(Length::Fill))
        .padding([4, 10])
        .width(width)
        .into()
}

fn cell(label: String, width: Length, align: Alignment) -> Element<'static, Message> {
    container(text(label).size(14).align_x(align).width(Length::Fill))
        .padding([5, 12])
        .width(width)
        .into()
}

fn num_cell(label: String, width: Length) -> Element<'static, Message> {
    cell(label, width, Alignment::End)
}

fn divider() -> Element<'static, Message> {
    container(space::horizontal())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|theme| container::Style {
            background: Some(
                match theme {
                    Theme::Light => iced::Color::from_rgba(0.80, 0.80, 0.86, 1.0),
                    _ => iced::Color::from_rgba(0.20, 0.20, 0.26, 1.0),
                }
                .into(),
            ),
            ..Default::default()
        })
        .into()
}

fn filter_bar<'a>(model: &'a AppModel, placeholder: &str) -> Element<'a, Message> {
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

fn source_name(object_path: &str) -> String {
    let inner = if let Some(p) = object_path.find('(') {
        &object_path[p + 1..object_path.len() - 1]
    } else {
        object_path
    };
    PathBuf::from(inner)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

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

pub fn files_view(model: &AppModel) -> Element<'_, Message> {
    if model.module_groups.is_empty() {
        return empty_view(model);
    }

    let size_label = mk_label(&fl!("column-size"), sort_indicator(model));

    let header = row![
        space::horizontal(),
        hdr_lbl(fl!("column-module"), Length::Fixed(200.0), Alignment::Start),
        hdr_btn(size_label, Length::Fixed(130.0), Message::SortGroup),
        hdr_lbl(fl!("symbols"), Length::Fixed(80.0), Alignment::End),
    ];

    let q = model.search_query.to_lowercase();
    let filtered: Vec<&GroupSummary> = model
        .module_groups
        .iter()
        .filter(|g| {
            q.is_empty()
                || g.name.to_lowercase().contains(&q)
                || source_name(&g.name).to_lowercase().contains(&q)
                || archive_name(&g.name).to_lowercase().contains(&q)
        })
        .collect();

    let mut rows = Vec::with_capacity(filtered.len() * 2);
    for g in &filtered {
        rows.push(divider());
        let src = source_name(&g.name);
        let arc = archive_name(&g.name);
        let item = button(row![
            cell(src, Length::Fill, Alignment::Start),
            cell(arc, Length::Fixed(200.0), Alignment::Start),
            num_cell(format_size(g.total_size), Length::Fixed(130.0)),
            num_cell(g.num_symbols.to_string(), Length::Fixed(80.0)),
        ])
        .on_press(Message::DrillInto(g.name.clone()))
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

pub fn modules_view(model: &AppModel) -> Element<'_, Message> {
    if model.archive_groups.is_empty() {
        return empty_view(model);
    }

    let size_label = mk_label(&fl!("column-size"), sort_indicator(model));

    let header = row![
        space::horizontal(),
        hdr_btn(size_label, Length::Fixed(130.0), Message::SortGroup),
        hdr_lbl(fl!("symbols"), Length::Fixed(80.0), Alignment::End),
    ];

    let q = model.search_query.to_lowercase();
    let filtered: Vec<&GroupSummary> = model
        .archive_groups
        .iter()
        .filter(|g| q.is_empty() || g.name.to_lowercase().contains(&q))
        .collect();

    let mut rows = Vec::with_capacity(filtered.len() * 2);
    for g in &filtered {
        rows.push(divider());
        let display_name = PathBuf::from(&g.name)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| g.name.clone());
        let item = button(row![
            cell(display_name, Length::Fill, Alignment::Start),
            num_cell(format_size(g.total_size), Length::Fixed(130.0)),
            num_cell(g.num_symbols.to_string(), Length::Fixed(80.0)),
        ])
        .on_press(Message::DrillInto(g.name.clone()))
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

pub fn sections_view(model: &AppModel) -> Element<'_, Message> {
    if model.section_categories.is_empty() {
        return empty_view(model);
    }

    let header = row![
        space::horizontal(),
        hdr_lbl(fl!("column-size"), Length::Fixed(130.0), Alignment::End),
        hdr_lbl(fl!("symbols"), Length::Fixed(80.0), Alignment::End),
    ];

    let mut rows: Vec<Element<'_, Message>> =
        Vec::with_capacity(model.section_categories.len() * 2);
    for g in &model.section_categories {
        rows.push(divider());
        let is_open = model.expanded_section.as_deref() == Some(&g.name);
        let arrow = if is_open { "\u{25BC}" } else { "\u{25B6}" };

        let cat_row = button(
            row![
                text(format!("{arrow}  {}", g.name))
                    .size(14)
                    .width(Length::Fill),
                num_cell(format_size(g.total_size), Length::Fixed(130.0)),
                num_cell(g.num_symbols.to_string(), Length::Fixed(80.0)),
            ]
            .padding(0),
        )
        .on_press(Message::ToggleSection(g.name.clone()))
        .width(Length::Fill)
        .style(button::text)
        .padding(0);
        rows.push(cat_row.into());

        if is_open {
            let mut sub_rows: Vec<Element<'_, Message>> = Vec::new();
            for sg in &model.section_groups {
                if matches_category(&g.name, &sg.name) {
                    sub_rows.push(
                        row![
                            cell(format!("    {}", sg.name), Length::Fill, Alignment::Start),
                            num_cell(format_size(sg.total_size), Length::Fixed(130.0)),
                            num_cell(sg.num_symbols.to_string(), Length::Fixed(80.0)),
                        ]
                        .into(),
                    );
                }
            }
            if !sub_rows.is_empty() {
                rows.push(column(sub_rows).spacing(1).into());
            }
        }
    }

    let body = scrollable(column(rows).spacing(0)).height(Length::Fill);

    column![header, divider(), body]
        .spacing(0)
        .padding([0, 10])
        .height(Length::Fill)
        .into()
}

pub fn symbols_view(model: &AppModel) -> Element<'_, Message> {
    let base = model.base_entries();
    let by_group = if let Some(ref group) = model.drilldown_group {
        filter_by_module(&base, group)
    } else {
        base
    };

    let mut filtered = filter_entries(&by_group, &model.search_query);
    sort_entries(&mut filtered, SortColumn::Size, model.sort_ascending);

    if filtered.is_empty() && model.all_entries.is_empty() {
        return empty_view(model);
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
                    text("\u{2190}").size(13),
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

    let hsz = mk_label(&fl!("column-size"), sort_indicator(model));

    let header = row![
        space::horizontal(),
        hdr_btn(hsz, Length::Fixed(110.0), Message::SortBy(SortColumn::Size)),
    ];
    content = content.push(header);

    let total = model.summary.total_size.max(1);
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
                    Length::Fixed(150.0),
                    Alignment::End,
                ),
                num_cell(format_size(entry.size), Length::Fixed(110.0)),
                num_cell(format!("{pct:.2}%"), Length::Fixed(80.0)),
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

pub fn summary_view(model: &AppModel) -> Element<'_, Message> {
    if model.all_entries.is_empty() {
        return empty_view(model);
    }

    let s = &model.summary;

    let file_info: Element<'_, Message> = if let Some(path) = &model.file_path {
        text(format!("File: {}", path.display())).into()
    } else {
        text("").into()
    };

    let total_lbl = fl!("total-size");
    let code_lbl = fl!("code-size");
    let data_lbl = fl!("data-size");
    let bss_lbl = fl!("bss-size");
    let other_lbl = fl!("label-other");
    let sym_lbl = fl!("symbols");
    let files_lbl = fl!("label-object-files");
    let sec_lbl = fl!("label-section-types");

    let items: Vec<(String, String)> = vec![
        (total_lbl, format_size(s.total_size)),
        (code_lbl, format_size(s.text_size)),
        (data_lbl, format_size(s.data_size + s.rodata_size)),
        (bss_lbl, format_size(s.bss_size)),
        (other_lbl, format_size(s.other_size)),
        (sym_lbl, s.num_symbols.to_string()),
        (files_lbl, s.num_files.to_string()),
        (sec_lbl, s.num_sections.to_string()),
    ];

    let rows: Vec<Element<'_, Message>> = items
        .into_iter()
        .map(|(l, v)| {
            row![text(l).width(Length::Fixed(180.0)), text(v).size(13)]
                .padding([4, 0])
                .into()
        })
        .collect();

    let section_title = container(text(fl!("summary")).size(20)).padding([0, 10]);

    column![file_info, section_title, column(rows).spacing(10)]
        .spacing(20)
        .padding([0, 10])
        .height(Length::Fill)
        .into()
}

pub fn empty_view(model: &AppModel) -> Element<'_, Message> {
    let msg: Element<'_, Message> = if let Some(err) = &model.error {
        column![
            text("\u{26A0}").size(48),
            text(fl!("error-loading")).size(24),
            text(err.clone()).size(14),
        ]
        .spacing(10)
        .align_x(Alignment::Center)
        .into()
    } else {
        column![
            text("\u{1F4E6}").size(64),
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

fn filter_by_module(entries: &[MapEntry], filepath: &str) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| {
            let ep = e.filepath.to_string_lossy();
            if ep == filepath {
                return true;
            }
            if let Some(p) = ep.find('(') {
                return &ep[..p] == filepath;
            }
            false
        })
        .cloned()
        .collect()
}
