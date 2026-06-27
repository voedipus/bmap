//! Application state and the root GPUI entity.

use crate::i18n::{t, Language};
use crate::model::*;
use crate::theme;
use crate::ui::input::{InputEvent, SearchInput};
use crate::ui::pages;
use crate::ui::toolbar;

use gpui::prelude::*;
use gpui::*;
use mapfile_parser::MapFile;
use std::path::PathBuf;

pub struct BmapApp {
    pub current_page: Page,
    pub language: Language,
    pub file_path: Option<PathBuf>,
    pub error: Option<String>,
    pub all_entries: Vec<MapEntry>,
    pub module_groups: Vec<GroupSummary>,
    pub archive_groups: Vec<GroupSummary>,
    pub section_groups: Vec<GroupSummary>,
    pub section_categories: Vec<GroupSummary>,
    pub summary: FileSummary,
    pub search_query: SharedString,
    pub show_debug: bool,
    pub sort_ascending: bool,
    pub drilldown_group: Option<SharedString>,
    pub expanded_section: Option<SharedString>,
    pub search_input: Entity<SearchInput>,
    _search_subscription: Subscription,
}

/// Top-level pages shown in the toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Files,
    Modules,
    Sections,
    Summary,
}

impl BmapApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let language = Language::detect();
        let placeholder = t("search-placeholder", language);
        let search_input = cx.new(|cx| SearchInput::new(placeholder, cx));
        let subscription = cx.subscribe(&search_input, |this, _input, event, cx| {
            let InputEvent::Changed(text) = event;
            this.search_query = text.clone();
            cx.notify();
        });

        Self {
            current_page: Page::Files,
            language,
            file_path: None,
            error: None,
            all_entries: Vec::new(),
            module_groups: Vec::new(),
            archive_groups: Vec::new(),
            section_groups: Vec::new(),
            section_categories: Vec::new(),
            summary: FileSummary::default(),
            search_query: "".into(),
            show_debug: false,
            sort_ascending: false,
            drilldown_group: None,
            expanded_section: None,
            search_input,
            _search_subscription: subscription,
        }
    }

    pub fn open_file(&mut self, cx: &mut Context<Self>) {
        let language = self.language;
        cx.spawn(async move |this, cx| {
            let result = rfd::AsyncFileDialog::new()
                .set_title(t("open-map-file", language).to_string())
                .add_filter("MAP files", &["map"])
                .add_filter("All files", &["*"])
                .pick_file()
                .await;
            if let Some(handle) = result {
                let path = handle.path().to_path_buf();
                let path_for_error = path.clone();
                let read_result = cx
                    .background_executor()
                    .spawn(async move { std::fs::read_to_string(&path) })
                    .await;
                match read_result {
                    Ok(contents) => {
                        this.update(cx, |app, cx| {
                            app.load_file(path_for_error, contents, cx);
                            cx.notify();
                        })?;
                    }
                    Err(why) => {
                        this.update(cx, |app, cx| {
                            app.error = Some(format!(
                                "failed to read {}: {why}",
                                path_for_error.display()
                            ));
                            cx.notify();
                        })?;
                    }
                }
            }
            Ok::<_, anyhow::Error>(())
        })
        .detach_and_log_err(cx);
    }

    fn load_file(&mut self, path: PathBuf, contents: String, cx: &mut Context<Self>) {
        self.file_path = Some(path);
        self.error = None;
        let map = MapFile::new_from_map_str(&contents);
        self.all_entries = build_symbol_entries(&map);
        self.recompute_groups();
        self.drilldown_group = None;
        self.expanded_section = None;
        self.search_query = "".into();
        self.search_input
            .update(cx, |input, cx| input.set_text("", cx));
        cx.notify();
    }

    fn recompute_groups(&mut self) {
        let entries = self.visible_entries();
        self.module_groups = group_by_module(&entries);
        self.archive_groups = group_by_archive(&entries);
        self.section_groups = group_by_section(&entries);
        self.section_categories = group_section_categories(&entries);
        self.summary = compute_file_summary(&entries);
    }

    pub fn visible_entries(&self) -> Vec<MapEntry> {
        if self.show_debug {
            self.all_entries.clone()
        } else {
            filter_debug_entries(&self.all_entries)
        }
    }

    pub fn select_page(&mut self, page: Page, cx: &mut Context<Self>) {
        self.current_page = page;
        self.drilldown_group = None;
        self.expanded_section = None;
        self.search_query = "".into();
        self.search_input
            .update(cx, |input, cx| input.set_text("", cx));
        cx.notify();
    }

    pub fn toggle_debug(&mut self, cx: &mut Context<Self>) {
        self.show_debug = !self.show_debug;
        self.recompute_groups();
        cx.notify();
    }

    pub fn toggle_group_sort(&mut self, cx: &mut Context<Self>) {
        self.sort_ascending = !self.sort_ascending;
        sort_groups(&mut self.module_groups, self.sort_ascending);
        sort_groups(&mut self.archive_groups, self.sort_ascending);
        sort_groups(&mut self.section_groups, self.sort_ascending);
        cx.notify();
    }

    pub fn drill_into(&mut self, group: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.drilldown_group = Some(group.into());
        self.search_query = "".into();
        self.search_input
            .update(cx, |input, cx| input.set_text("", cx));
        cx.notify();
    }

    pub fn drill_out(&mut self, cx: &mut Context<Self>) {
        self.drilldown_group = None;
        self.search_query = "".into();
        self.search_input
            .update(cx, |input, cx| input.set_text("", cx));
        cx.notify();
    }

    pub fn toggle_section(&mut self, name: impl Into<SharedString>, cx: &mut Context<Self>) {
        let name = name.into();
        if self.expanded_section.as_ref() == Some(&name) {
            self.expanded_section = None;
        } else {
            self.expanded_section = Some(name);
        }
        cx.notify();
    }
}

impl Render for BmapApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::BG_PRIMARY)
            .text_color(theme::TEXT_PRIMARY)
            .child(toolbar::render(self, cx))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(pages::render(self, cx)),
            )
    }
}

pub fn run() {
    gpui_platform::application().run(|cx| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(100.0),
                        y: px(100.0),
                    },
                    size: Size {
                        width: px(960.0),
                        height: px(640.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("bmap".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(BmapApp::new),
        )
        .unwrap();
    });
}
