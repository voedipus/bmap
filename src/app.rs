//! Application state, messages, and the Elm-style update/view loop.

use crate::fl;
use crate::model::*;

use iced::{Element, Length, Task, Theme};
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
    pub current_page: Page,

    pub all_entries: Vec<MapEntry>,
    pub module_groups: Vec<GroupSummary>,
    pub archive_groups: Vec<GroupSummary>,
    pub section_groups: Vec<GroupSummary>,
    pub section_categories: Vec<GroupSummary>,
    pub summary: FileSummary,
    pub file_path: Option<PathBuf>,
    pub error: Option<String>,

    pub search_query: String,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub show_debug: bool,

    pub drilldown_group: Option<String>,
    pub expanded_section: Option<String>,
}

#[derive(Debug, Clone)]
/// User actions and async results handled by the application update loop.
pub enum Message {
    OpenFile,
    FileLoaded(PathBuf, String),
    FileError(String),
    SearchChanged(String),
    SortBy(SortColumn),
    ToggleGroupSort,
    SelectPage(Page),
    ToggleDebug(bool),
    DrillInto(String),
    DrillOut,
    ToggleSection(String),
}

impl AppModel {
    pub fn new() -> (Self, Task<Message>) {
        let app = AppModel {
            current_page: Page::Files,
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
        let mut title = fl!("app-title");
        if let Some(p) = &self.file_path {
            title.push_str(" — ");
            title.push_str(&p.file_name().unwrap_or_default().to_string_lossy());
        }
        title
    }

    pub fn theme(&self) -> Theme {
        match dark_light::detect() {
            Ok(dark_light::Mode::Light) => Theme::Light,
            _ => Theme::custom(
                std::borrow::Cow::Borrowed("bmap-dark"),
                iced::theme::Palette {
                    background: iced::Color::from_rgba(0.08, 0.08, 0.11, 1.0),
                    text: iced::Color::from_rgba(0.88, 0.88, 0.94, 1.0),
                    primary: iced::Color::from_rgba(0.39, 0.40, 0.95, 1.0),
                    success: iced::Color::from_rgba(0.2, 0.8, 0.4, 1.0),
                    warning: iced::Color::from_rgba(0.9, 0.7, 0.2, 1.0),
                    danger: iced::Color::from_rgba(0.9, 0.3, 0.3, 1.0),
                },
            ),
        }
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
                self.file_path = Some(path);
                self.error = None;
                let map = MapFile::new_from_map_str(&contents);
                self.all_entries = build_symbol_entries(&map);
                self.recompute_groups();
                self.drilldown_group = None;
                self.expanded_section = None;
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
            Message::ToggleGroupSort => {
                self.sort_ascending = !self.sort_ascending;
                sort_groups(&mut self.module_groups, self.sort_ascending);
                sort_groups(&mut self.archive_groups, self.sort_ascending);
                sort_groups(&mut self.section_groups, self.sort_ascending);
            }
            Message::SelectPage(page) => {
                self.current_page = page;
                self.drilldown_group = None;
                self.expanded_section = None;
                self.search_query.clear();
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
            Page::Files => self.group_or_symbols_view(),
            Page::Modules => self.group_or_symbols_view(),
            Page::Sections => crate::views::sections_view(self),
            Page::Summary => crate::views::summary_view(self),
        };

        iced::widget::column![crate::views::toolbar(self), content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Shows either the group list or the symbol drill-down for pages that support drill-down.
    fn group_or_symbols_view(&self) -> Element<'_, Message> {
        if self.drilldown_group.is_some() {
            crate::views::symbols_view(self)
        } else if self.current_page == Page::Files {
            crate::views::source_files_view(self)
        } else {
            crate::views::modules_view(self)
        }
    }

    /// Returns the entries currently visible, respecting the debug-symbol toggle.
    pub fn visible_entries(&self) -> Vec<MapEntry> {
        if self.show_debug {
            self.all_entries.clone()
        } else {
            filter_debug_entries(&self.all_entries)
        }
    }

    fn recompute_groups(&mut self) {
        let entries = self.visible_entries();
        self.module_groups = group_by_module(&entries);
        self.archive_groups = group_by_archive(&entries);
        self.section_groups = group_by_section(&entries);
        self.section_categories = group_section_categories(&entries);
        self.summary = compute_file_summary(&entries);
    }
}

/// Top-level pages shown in the toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Files,
    Modules,
    Sections,
    Summary,
}
