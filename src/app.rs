// SPDX-License-Identifier: MPL-2.0

use crate::fl;
use crate::model::*;

use iced::widget::{button, checkbox, column, container, row, scrollable, space, text, text_input};
use iced::{Alignment, Element, Length, Task, Theme};
use mapfile_parser::MapFile;
use std::path::PathBuf;

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

pub struct AppModel {
    window_title: String,
    current_page: Page,

    // MAP file data
    #[allow(dead_code)]
    map_file: Option<MapFile>,
    all_entries: Vec<MapEntry>,
    filtered_entries: Vec<MapEntry>,
    module_groups: Vec<GroupSummary>,
    section_groups: Vec<GroupSummary>,
    summary: FileSummary,
    file_path: Option<PathBuf>,
    error: Option<String>,

    // View state
    search_query: String,
    sort_column: SortColumn,
    sort_ascending: bool,
    show_debug: bool,

    // Drill-down state
    drilldown_group: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
    FileLoaded(PathBuf, String),
    FileError(String),
    SearchChanged(String),
    SortBy(SortColumn),
    SortGroup,
    SelectPage(Page),
    ToggleDebug(bool),
    DrillInto(String),
    DrillOut,
}

impl AppModel {
    pub fn new() -> (Self, Task<Message>) {
        let app = AppModel {
            window_title: fl!("app-title"),
            current_page: Page::AllSymbols,
            map_file: None,
            all_entries: Vec::new(),
            filtered_entries: Vec::new(),
            module_groups: Vec::new(),
            section_groups: Vec::new(),
            summary: FileSummary::default(),
            file_path: None,
            error: None,
            search_query: String::new(),
            sort_column: SortColumn::Size,
            sort_ascending: false,
            show_debug: false,
            drilldown_group: None,
        };
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        self.window_title.clone()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                return Task::perform(
                    async {
                        let dialog = rfd::AsyncFileDialog::new()
                            .set_title(&fl!("open-map-file"))
                            .add_filter("MAP files", &["map"])
                            .add_filter("All files", &["*"]);

                        match dialog.pick_file().await {
                            Some(handle) => {
                                let path = handle.path().to_path_buf();
                                match tokio::fs::read_to_string(&path).await {
                                    Ok(contents) => Message::FileLoaded(path, contents),
                                    Err(why) => Message::FileError(format!(
                                        "failed to read {}: {why}",
                                        path.display()
                                    )),
                                }
                            }
                            None => Message::FileError(String::new()),
                        }
                    },
                    std::convert::identity,
                );
            }

            Message::FileLoaded(path, contents) => {
                self.file_path = Some(path.clone());
                self.error = None;
                let map = parse_map_file(&contents);
                self.all_entries = build_symbol_entries(&map);
                self.recompute_groups();
                self.map_file = Some(map);
                self.drilldown_group = None;
                self.update_filtered_entries();
                self.update_title();
            }

            Message::FileError(why) => {
                if !why.is_empty() {
                    self.error = Some(why);
                }
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
                self.update_filtered_entries();
            }

            Message::SortBy(col) => {
                if self.sort_column == col {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = col;
                    self.sort_ascending = false;
                }
                self.update_filtered_entries();
            }

            Message::SortGroup => {
                self.sort_ascending = !self.sort_ascending;
                sort_groups(&mut self.module_groups, self.sort_ascending);
                sort_groups(&mut self.section_groups, self.sort_ascending);
            }

            Message::SelectPage(page) => {
                self.current_page = page;
                self.drilldown_group = None;
                self.update_title();
            }

            Message::ToggleDebug(show) => {
                self.show_debug = show;
                self.recompute_groups();
                self.update_filtered_entries();
            }

            Message::DrillInto(group_name) => {
                self.drilldown_group = Some(group_name);
                self.update_filtered_entries();
            }

            Message::DrillOut => {
                self.drilldown_group = None;
                self.search_query.clear();
                self.update_filtered_entries();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<_> = match self.current_page {
            Page::AllSymbols => self.symbols_view(),
            Page::ByModule => {
                if self.drilldown_group.is_some() {
                    self.symbols_view()
                } else {
                    self.group_view(&self.module_groups, false)
                }
            }
            Page::BySection => {
                if self.drilldown_group.is_some() {
                    self.symbols_view()
                } else {
                    self.group_view(&self.section_groups, true)
                }
            }
            Page::Summary => self.summary_view(),
        };

        column![self.toolbar(), content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn toolbar(&self) -> Element<'_, Message> {
        let open_btn = button(text(fl!("open")))
            .on_press(Message::OpenFile)
            .padding([4, 12]);

        let debug_toggle = row![
            checkbox(self.show_debug)
                .on_toggle(Message::ToggleDebug)
                .size(14),
            text("Debug").size(13),
        ]
        .align_y(Alignment::Center)
        .spacing(4);

        let pages = [
            (Page::AllSymbols, "\u{2630}", fl!("all-symbols")),
            (Page::ByModule, "\u{1f4c1}", fl!("by-module")),
            (Page::BySection, "\u{2195}", fl!("by-section")),
            (Page::Summary, "\u{2261}", fl!("summary")),
        ];

        let nav_buttons: Vec<Element<'_, Message>> = pages
            .into_iter()
            .map(|(page, icon, label)| {
                let is_active = self.current_page == page;
                let mut btn = button(
                    row![text(icon).size(14), text(label).size(13)]
                        .align_y(Alignment::Center)
                        .spacing(4)
                        .padding([2, 8]),
                );
                if !is_active {
                    btn = btn.style(button::secondary);
                }
                btn.on_press(Message::SelectPage(page)).into()
            })
            .collect();

        let filename = if let Some(p) = &self.file_path {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };

        container(
            row![
                open_btn,
                debug_toggle,
                space::horizontal().width(Length::Fixed(8.0)),
                row(nav_buttons).spacing(4),
                space::horizontal(),
                text(filename).size(13),
            ]
            .align_y(Alignment::Center)
            .spacing(8)
            .padding([6, 10]),
        )
        .style(|_theme| container::Style {
            background: Some(iced::Color::from_rgba(0.08, 0.08, 0.08, 1.0).into()),
            ..container::Style::default()
        })
        .width(Length::Fill)
        .into()
    }

    fn recompute_groups(&mut self) {
        let entries: Vec<MapEntry> = if self.show_debug {
            self.all_entries.clone()
        } else {
            filter_debug_entries(&self.all_entries)
        };
        self.module_groups = group_by_module(&entries);
        self.section_groups = group_by_section(&entries);
        self.summary = compute_file_summary(&entries);
    }

    fn update_title(&mut self) {
        let mut title = fl!("app-title");
        if let Some(p) = &self.file_path {
            title.push_str(" — ");
            title.push_str(&p.file_name().unwrap_or_default().to_string_lossy());
        }
        self.window_title = title;
    }

    fn update_filtered_entries(&mut self) {
        let base_entries: Vec<MapEntry> = if self.show_debug {
            self.all_entries.clone()
        } else {
            filter_debug_entries(&self.all_entries)
        };

        // Apply drill-down filter first
        let entries = if let Some(ref group) = self.drilldown_group {
            match self.current_page {
                Page::ByModule => filter_by_module(&base_entries, group),
                Page::BySection => filter_by_section(&base_entries, group),
                _ => base_entries,
            }
        } else {
            base_entries
        };

        let mut filtered = filter_entries(&entries, &self.search_query);
        sort_entries(&mut filtered, self.sort_column, self.sort_ascending);
        self.filtered_entries = filtered;
    }

    fn sort_indicator(&self, col: SortColumn) -> &'static str {
        if self.sort_column == col {
            if self.sort_ascending {
                "  \u{25B2}"
            } else {
                "  \u{25BC}"
            }
        } else {
            ""
        }
    }

    fn make_label(base: &str, indicator: &str) -> String {
        format!("{base}{indicator}")
    }

    fn header_cell_button(
        label: String,
        width: Length,
        on_press: Message,
    ) -> Element<'static, Message> {
        button(text(label).size(14).width(Length::Fill))
            .width(width)
            .on_press(on_press)
            .padding([4, 8])
            .style(button::text)
            .into()
    }

    fn header_cell_label(label: String, width: Length) -> Element<'static, Message> {
        container(text(label).size(14))
            .padding([4, 8])
            .width(width)
            .into()
    }

    fn data_cell(label: String, width: Length) -> Element<'static, Message> {
        container(text(label).size(14).width(width).height(Length::Shrink))
            .padding([2, 8])
            .into()
    }

    fn symbols_view(&self) -> Element<'_, Message> {
        if self.filtered_entries.is_empty() && self.all_entries.is_empty() {
            return self.empty_view();
        }

        // Breadcrumb / back button when drilled in
        let mut content = column![].spacing(4);

        if let Some(ref group) = self.drilldown_group {
            let short_name = PathBuf::from(group)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| group.clone());
            content = content.push(
                button(
                    row![
                        text("\u{2190}").size(14),
                        text(format!("Back to groups ({short_name})")).size(14)
                    ]
                    .align_y(Alignment::Center)
                    .spacing(6),
                )
                .on_press(Message::DrillOut)
                .style(button::text)
                .padding([4, 0]),
            );
        }

        let search = text_input(&fl!("search-placeholder"), &self.search_query)
            .on_input(Message::SearchChanged)
            .width(Length::Fill)
            .padding(8);
        content = content.push(search);

        let ind = |col: SortColumn| self.sort_indicator(col);
        let h3 = Self::make_label(&fl!("column-size"), ind(SortColumn::Size));

        let header = row![
            Self::header_cell_label(fl!("column-name"), Length::Fill),
            Self::header_cell_label(fl!("column-address"), Length::Fixed(160.0)),
            Self::header_cell_button(h3, Length::Fixed(120.0), Message::SortBy(SortColumn::Size)),
            Self::header_cell_label(fl!("column-percentage"), Length::Fixed(90.0)),
        ];
        content = content.push(header);

        let total = self.summary.total_size.max(1);

        let row_count = self.filtered_entries.len().min(5000);
        let mut rows = Vec::with_capacity(row_count);
        for entry in self.filtered_entries.iter().take(row_count) {
            let pct = if total > 0 {
                (entry.size as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let item = row![
                Self::data_cell(entry.name.clone(), Length::Fill),
                Self::data_cell(format!("0x{:08X}", entry.address), Length::Fixed(160.0)),
                Self::data_cell(format_size(entry.size), Length::Fixed(120.0)),
                Self::data_cell(format!("{pct:.2}%"), Length::Fixed(90.0)),
            ];
            rows.push(item.into());
        }

        let body: Element<'_, Message> = if rows.is_empty() {
            text("No matching symbols.").into()
        } else {
            scrollable(column(rows).spacing(1))
                .height(Length::Fill)
                .into()
        };
        content = content.push(body);

        content.padding(8).height(Length::Fill).into()
    }

    fn group_view(&self, groups: &[GroupSummary], is_section: bool) -> Element<'_, Message> {
        if groups.is_empty() {
            return self.empty_view();
        }

        let name_label =
            Self::make_label(&fl!("column-name"), self.sort_indicator(SortColumn::Name));
        let size_label =
            Self::make_label(&fl!("column-size"), self.sort_indicator(SortColumn::Size));

        let header = row![
            Self::header_cell_button(name_label, Length::Fill, Message::SortGroup),
            Self::header_cell_button(size_label, Length::Fixed(140.0), Message::SortGroup),
            Self::header_cell_label(fl!("symbols"), Length::Fixed(90.0)),
        ];

        let mut rows = Vec::with_capacity(groups.len());
        for g in groups {
            let display_name = if is_section {
                g.name.clone()
            } else {
                PathBuf::from(&g.name)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| g.name.clone())
            };
            // Use the full group name as the drill-down key
            let item = button(row![
                Self::data_cell(display_name.clone(), Length::Fill),
                Self::data_cell(format_size(g.total_size), Length::Fixed(140.0)),
                Self::data_cell(g.num_symbols.to_string(), Length::Fixed(90.0)),
            ])
            .on_press(Message::DrillInto(g.name.clone()))
            .padding(0)
            .width(Length::Fill)
            .style(button::text);
            rows.push(item.into());
        }

        let body = scrollable(column(rows).spacing(1)).height(Length::Fill);

        column![header, body]
            .spacing(4)
            .padding(8)
            .height(Length::Fill)
            .into()
    }

    fn empty_view(&self) -> Element<'_, Message> {
        let msg: Element<'_, Message> = if let Some(err) = &self.error {
            column![
                text(fl!("error-loading")).size(24),
                text(err.clone()).size(16),
            ]
            .spacing(8)
            .align_x(Alignment::Center)
            .into()
        } else {
            column![
                text(fl!("no-file-loaded")).size(24),
                text("Click \"Open\" to load a linker MAP file.").size(16),
            ]
            .spacing(8)
            .align_x(Alignment::Center)
            .into()
        };

        container(msg)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn summary_view(&self) -> Element<'_, Message> {
        if self.all_entries.is_empty() {
            return self.empty_view();
        }

        let s = &self.summary;

        let file_info: Element<'_, Message> = if let Some(path) = &self.file_path {
            text(format!("File: {}", path.display())).into()
        } else {
            text("").into()
        };

        let summary_items = vec![
            (fl!("total-size"), format_size(s.total_size)),
            (fl!("code-size"), format_size(s.text_size)),
            (fl!("data-size"), format_size(s.data_size + s.rodata_size)),
            (fl!("bss-size"), format_size(s.bss_size)),
            ("Other".to_string(), format_size(s.other_size)),
            ("Symbols".to_string(), s.num_symbols.to_string()),
            ("Files".to_string(), s.num_files.to_string()),
            ("Section types".to_string(), s.num_sections.to_string()),
        ];

        let items: Vec<Element<'_, Message>> = summary_items
            .into_iter()
            .map(|(label, value)| {
                row![text(label).width(Length::Fixed(160.0)), text(value),]
                    .padding([4, 0])
                    .into()
            })
            .collect();

        let section_title = container(text(fl!("summary")).size(20)).padding([0, 8]);

        column![file_info, section_title, column(items).spacing(8)]
            .spacing(16)
            .padding(8)
            .height(Length::Fill)
            .into()
    }
}

/// Filter entries matching a specific module filepath.
fn filter_by_module(entries: &[MapEntry], filepath: &str) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| e.filepath.to_string_lossy() == filepath)
        .cloned()
        .collect()
}

/// Filter entries matching a specific section type.
fn filter_by_section(entries: &[MapEntry], section: &str) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| e.section_type == section)
        .cloned()
        .collect()
}

/// Filter out entries that belong to debug-related sections.
fn filter_debug_entries(entries: &[MapEntry]) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| !is_debug_section(&e.section_type))
        .cloned()
        .collect()
}

fn is_debug_section(section: &str) -> bool {
    section.starts_with(".debug_")
        || section == ".comment"
        || section.starts_with(".note")
        || section == ".stab"
        || section.starts_with(".stabstr")
        || section.starts_with(".ARM.attributes")
        || section.starts_with(".ARM.exidx")
        || section.starts_with(".ARM.extab")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    AllSymbols,
    ByModule,
    BySection,
    Summary,
}
