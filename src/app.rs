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

    #[allow(dead_code)]
    map_file: Option<MapFile>,
    all_entries: Vec<MapEntry>,
    module_groups: Vec<GroupSummary>,
    archive_groups: Vec<GroupSummary>,
    section_groups: Vec<GroupSummary>,
    section_categories: Vec<GroupSummary>,
    summary: FileSummary,
    file_path: Option<PathBuf>,
    error: Option<String>,

    search_query: String,
    sort_column: SortColumn,
    sort_ascending: bool,
    show_debug: bool,

    drilldown_group: Option<String>,
    expanded_section: Option<String>,
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
    ToggleSection(String),
}

impl AppModel {
    pub fn new() -> (Self, Task<Message>) {
        let app = AppModel {
            window_title: fl!("app-title"),
            current_page: Page::Files,
            map_file: None,
            all_entries: Vec::new(),
            module_groups: Vec::new(),
            archive_groups: Vec::new(),
            section_groups: Vec::new(),
            section_categories: Vec::new(),
            summary: FileSummary::default(),
            file_path: None,
            error: None,
            search_query: String::new(),
            sort_column: SortColumn::Size,
            sort_ascending: false,
            show_debug: false,
            drilldown_group: None,
            expanded_section: None,
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
                self.expanded_section = None;
                self.update_title();
            }
            Message::FileError(why) => {
                if !why.is_empty() {
                    self.error = Some(why);
                }
            }
            Message::SearchChanged(query) => {
                self.search_query = query;
            }
            Message::SortBy(col) => {
                if self.sort_column == col {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = col;
                    self.sort_ascending = false;
                }
            }
            Message::SortGroup => {
                self.sort_ascending = !self.sort_ascending;
                sort_groups(&mut self.module_groups, self.sort_ascending);
                sort_groups(&mut self.section_groups, self.sort_ascending);
            }
            Message::SelectPage(page) => {
                self.current_page = page;
                self.drilldown_group = None;
                self.expanded_section = None;
                self.update_title();
            }
            Message::ToggleDebug(show) => {
                self.show_debug = show;
                self.recompute_groups();
            }
            Message::DrillInto(group_name) => {
                self.drilldown_group = Some(group_name);
            }
            Message::DrillOut => {
                self.drilldown_group = None;
                self.search_query.clear();
            }
            Message::ToggleSection(name) => {
                if self.expanded_section.as_deref() == Some(&name) {
                    self.expanded_section = None;
                } else {
                    self.expanded_section = Some(name);
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<_> = match self.current_page {
            Page::Files => {
                if self.drilldown_group.is_some() {
                    self.symbols_view()
                } else {
                    self.files_view()
                }
            }
            Page::Modules => {
                if self.drilldown_group.is_some() {
                    self.symbols_view()
                } else {
                    self.modules_view()
                }
            }
            Page::Sections => self.sections_view(),
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
            text(fl!("label-debug")).size(13),
        ]
        .align_y(Alignment::Center)
        .spacing(4);

        let pages = [
            (Page::Files, "\u{1f4e6}", fl!("files")),
            (Page::Modules, "\u{1f4c1}", fl!("modules")),
            (Page::Sections, "\u{2195}", fl!("sections")),
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

        container(
            row![
                open_btn,
                space::horizontal().width(Length::Fixed(8.0)),
                row(nav_buttons).spacing(4),
                space::horizontal(),
                debug_toggle,
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
        self.archive_groups = group_by_archive(&entries);
        self.section_groups = group_by_section(&entries);
        self.section_categories = group_section_categories(&entries);
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

    fn filtered_base_entries(&self) -> Vec<MapEntry> {
        if self.show_debug {
            self.all_entries.clone()
        } else {
            filter_debug_entries(&self.all_entries)
        }
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

    fn table_divider() -> Element<'static, Message> {
        container(space::horizontal())
            .height(Length::Fixed(1.0))
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Color::from_rgba(0.22, 0.22, 0.28, 1.0).into()),
                ..Default::default()
            })
            .into()
    }

    // ── Files View ───────────────────────────────────────────

    fn files_view(&self) -> Element<'_, Message> {
        if self.module_groups.is_empty() {
            return self.empty_view();
        }

        let search = text_input(&fl!("search-by-source"), &self.search_query)
            .on_input(Message::SearchChanged)
            .width(Length::Fill)
            .padding(8);

        let size_label =
            Self::make_label(&fl!("column-size"), self.sort_indicator(SortColumn::Size));

        let header = row![
            Self::header_cell_label(fl!("column-source"), Length::Fill),
            Self::header_cell_label(fl!("column-module"), Length::Fixed(200.0)),
            Self::header_cell_button(size_label, Length::Fixed(140.0), Message::SortGroup),
            Self::header_cell_label(fl!("symbols"), Length::Fixed(90.0)),
        ];

        let q = self.search_query.to_lowercase();
        let filtered: Vec<&GroupSummary> = self
            .module_groups
            .iter()
            .filter(|g| {
                q.is_empty()
                    || g.name.to_lowercase().contains(&q)
                    || derive_source_name(&g.name).to_lowercase().contains(&q)
                    || derive_archive_name(&g.name).to_lowercase().contains(&q)
            })
            .collect();

        let mut rows = Vec::with_capacity(filtered.len() * 2);
        for g in &filtered {
            rows.push(Self::table_divider());
            let source = derive_source_name(&g.name);
            let module = derive_archive_name(&g.name);
            let item = button(row![
                Self::data_cell(source, Length::Fill),
                Self::data_cell(module, Length::Fixed(200.0)),
                Self::data_cell(format_size(g.total_size), Length::Fixed(140.0)),
                Self::data_cell(g.num_symbols.to_string(), Length::Fixed(90.0)),
            ])
            .on_press(Message::DrillInto(g.name.clone()))
            .padding(0)
            .width(Length::Fill)
            .style(button::text);
            rows.push(item.into());
        }

        let body = scrollable(column(rows).spacing(0)).height(Length::Fill);

        column![search, header, Self::table_divider(), body,]
            .spacing(4)
            .padding(8)
            .height(Length::Fill)
            .into()
    }

    // ── Modules View ─────────────────────────────────────────

    fn modules_view(&self) -> Element<'_, Message> {
        if self.archive_groups.is_empty() {
            return self.empty_view();
        }

        let search = text_input(&fl!("search-by-module"), &self.search_query)
            .on_input(Message::SearchChanged)
            .width(Length::Fill)
            .padding(8);

        let size_label =
            Self::make_label(&fl!("column-size"), self.sort_indicator(SortColumn::Size));

        let header = row![
            Self::header_cell_label(fl!("column-archive"), Length::Fill),
            Self::header_cell_button(size_label, Length::Fixed(140.0), Message::SortGroup),
            Self::header_cell_label(fl!("symbols"), Length::Fixed(90.0)),
        ];

        let q = self.search_query.to_lowercase();
        let filtered: Vec<&GroupSummary> = self
            .archive_groups
            .iter()
            .filter(|g| q.is_empty() || g.name.to_lowercase().contains(&q))
            .collect();

        let mut rows = Vec::with_capacity(filtered.len() * 2);
        for g in &filtered {
            rows.push(Self::table_divider());
            let display_name = PathBuf::from(&g.name)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| g.name.clone());
            let item = button(row![
                Self::data_cell(display_name, Length::Fill),
                Self::data_cell(format_size(g.total_size), Length::Fixed(140.0)),
                Self::data_cell(g.num_symbols.to_string(), Length::Fixed(90.0)),
            ])
            .on_press(Message::DrillInto(g.name.clone()))
            .padding(0)
            .width(Length::Fill)
            .style(button::text);
            rows.push(item.into());
        }

        let body = scrollable(column(rows).spacing(0)).height(Length::Fill);

        column![search, header, Self::table_divider(), body,]
            .spacing(4)
            .padding(8)
            .height(Length::Fill)
            .into()
    }

    // ── Sections View (table of categories + expandable subs) ──

    fn sections_view(&self) -> Element<'_, Message> {
        if self.section_categories.is_empty() {
            return self.empty_view();
        }

        let header = row![
            Self::header_cell_label(fl!("column-section"), Length::Fill),
            Self::header_cell_label(fl!("column-size"), Length::Fixed(140.0)),
            Self::header_cell_label(fl!("symbols"), Length::Fixed(90.0)),
        ];

        let mut rows = Vec::with_capacity(self.section_categories.len() * 2);
        for g in &self.section_categories {
            rows.push(Self::table_divider());
            let is_open = self.expanded_section.as_deref() == Some(&g.name);
            let arrow = if is_open { "\u{25BC}" } else { "\u{25B6}" };

            // Category row — click to expand
            let cat_row = button(
                row![
                    text(format!("{arrow}  {}", g.name))
                        .size(14)
                        .width(Length::Fill),
                    text(format_size(g.total_size))
                        .size(14)
                        .width(Length::Fixed(140.0)),
                    text(g.num_symbols.to_string())
                        .size(14)
                        .width(Length::Fixed(90.0)),
                ]
                .padding([4, 8]),
            )
            .on_press(Message::ToggleSection(g.name.clone()))
            .width(Length::Fill)
            .style(button::text);
            rows.push(cat_row.into());

            if is_open {
                // Show the sub-sections that belong to this category
                let sub_start = if g.name == "other" { "" } else { &g.name };
                let mut sub_rows: Vec<Element<'_, Message>> = Vec::new();
                for sg in &self.section_groups {
                    let matches = if g.name == "other" {
                        !sg.name.starts_with(".text")
                            && !sg.name.starts_with(".data")
                            && !sg.name.starts_with(".rodata")
                            && !sg.name.starts_with(".bss")
                            && !sg.name.starts_with(".sbss")
                    } else {
                        sg.name.starts_with(sub_start)
                    };
                    if matches {
                        sub_rows.push(
                            row![
                                Self::data_cell(format!("    {}", sg.name), Length::Fill),
                                Self::data_cell(format_size(sg.total_size), Length::Fixed(140.0),),
                                Self::data_cell(sg.num_symbols.to_string(), Length::Fixed(90.0),),
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

        column![header, Self::table_divider(), body,]
            .spacing(0)
            .padding(8)
            .height(Length::Fill)
            .into()
    }

    // ── Symbols View (drill-down) ─────────────────────────────

    fn symbols_view(&self) -> Element<'_, Message> {
        let base = self.filtered_base_entries();
        let filtered = if let Some(ref group) = self.drilldown_group {
            filter_by_module(&base, group)
        } else {
            base
        };

        let mut filtered2 = filter_entries(&filtered, &self.search_query);
        sort_entries(&mut filtered2, self.sort_column, self.sort_ascending);

        if filtered2.is_empty() && self.all_entries.is_empty() {
            return self.empty_view();
        }

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
                        text(format!("{} ({})", fl!("label-back-to-files"), short_name)).size(14)
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

        let h3 = Self::make_label(&fl!("column-size"), self.sort_indicator(SortColumn::Size));

        let header = row![
            Self::header_cell_label(fl!("column-name"), Length::Fill),
            Self::header_cell_label(fl!("column-address"), Length::Fixed(160.0)),
            Self::header_cell_button(h3, Length::Fixed(120.0), Message::SortBy(SortColumn::Size)),
            Self::header_cell_label(fl!("column-percentage"), Length::Fixed(90.0)),
        ];
        content = content.push(header);

        let total = self.summary.total_size.max(1);

        let row_count = filtered2.len().min(5000);
        let mut rows = Vec::with_capacity(row_count);
        for entry in filtered2.iter().take(row_count) {
            let pct = if total > 0 {
                (entry.size as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            rows.push(
                row![
                    Self::data_cell(entry.name.clone(), Length::Fill),
                    Self::data_cell(format!("0x{:08X}", entry.address), Length::Fixed(160.0),),
                    Self::data_cell(format_size(entry.size), Length::Fixed(120.0)),
                    Self::data_cell(format!("{pct:.2}%"), Length::Fixed(90.0)),
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
        content = content.push(body);

        content.padding(8).height(Length::Fill).into()
    }

    // ── Summary ──────────────────────────────────────────────

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
            (fl!("label-other"), format_size(s.other_size)),
            (fl!("symbols"), s.num_symbols.to_string()),
            (fl!("label-object-files"), s.num_files.to_string()),
            (fl!("label-section-types"), s.num_sections.to_string()),
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

    // ── Empty ────────────────────────────────────────────────

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
                text(fl!("open-instruction")).size(16),
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
}

// ── Helpers ─────────────────────────────────────────────────

fn derive_source_name(object_path: &str) -> String {
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

fn derive_archive_name(object_path: &str) -> String {
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

fn filter_by_module(entries: &[MapEntry], filepath: &str) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| {
            let ep = e.filepath.to_string_lossy();
            // Exact match (for source file drill-down)
            if ep == filepath {
                return true;
            }
            // Archive match: "libfoo.a" matches "libfoo.a(main.c.o)"
            if let Some(p) = ep.find('(') {
                return &ep[..p] == filepath;
            }
            false
        })
        .cloned()
        .collect()
}

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
    Files,
    Modules,
    Sections,
    Summary,
}
