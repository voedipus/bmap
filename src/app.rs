//! Application state and root view.

use crate::i18n::{t, Language};
use crate::model::*;
use crate::ui::pages;
use crate::ui::toolbar;

use gpui::prelude::*;
use gpui::*;
use gpui_component::input::{InputEvent, InputState};
use gpui_component::ActiveTheme;
use mapfile_parser::MapFile;
use std::path::PathBuf;

/// Top-level pages shown in the toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Files,
    Modules,
    Sections,
    Summary,
}

pub struct BmapApp {
    pub current_page: Page,
    pub language: Language,
    pub file_path: Option<PathBuf>,
    pub error: Option<String>,
    pub all_entries: Vec<MapEntry>,
    pub aggregate: Aggregate,
    pub show_debug: bool,
    pub sort_ascending: bool,
    pub drilldown_group: Option<String>,
    pub expanded_section: Option<String>,
    pub search_input: Entity<InputState>,
    pub search_query: String,
    _search_subscription: Subscription,
}

impl BmapApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let language = Language::detect();
        let search_state =
            cx.new(|cx| InputState::new(window, cx).placeholder(t("search-placeholder", language)));

        let subscription =
            cx.subscribe_in(&search_state, window, |this, input, event, _window, cx| {
                if let InputEvent::Change = event {
                    this.search_query = input.read(cx).value().to_string();
                    cx.notify();
                }
            });

        Self {
            current_page: Page::Files,
            language,
            file_path: None,
            error: None,
            all_entries: Vec::new(),
            aggregate: Aggregate::default(),
            show_debug: false,
            sort_ascending: false,
            drilldown_group: None,
            expanded_section: None,
            search_input: search_state,
            search_query: String::new(),
            _search_subscription: subscription,
        }
    }

    pub fn open_file(&mut self, cx: &mut Context<Self>) {
        let language = self.language;
        cx.spawn(
            move |this: gpui::WeakEntity<BmapApp>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    let result = rfd::AsyncFileDialog::new()
                        .set_title(t("open-map-file", language).to_string())
                        .add_filter("MAP files", &["map"])
                        .add_filter("All files", &["*"])
                        .pick_file()
                        .await;
                    if let Some(handle) = result {
                        let path = handle.path().to_path_buf();
                        let contents = cx
                            .background_executor()
                            .spawn({
                                let path = path.clone();
                                async move { std::fs::read_to_string(&path) }
                            })
                            .await;
                        match contents {
                            Ok(text) => {
                                this.update(&mut cx, |app, cx| {
                                    app.load_file(path, text, cx);
                                    cx.notify();
                                })
                                .ok();
                            }
                            Err(err) => {
                                let path_str = path.display().to_string();
                                this.update(&mut cx, move |app, cx| {
                                    app.error = Some(format!("failed to read {path_str}: {err}"));
                                    cx.notify();
                                })
                                .ok();
                            }
                        }
                    }
                }
            },
        )
        .detach();
    }

    fn load_file(&mut self, path: PathBuf, contents: String, cx: &mut Context<Self>) {
        self.file_path = Some(path);
        self.error = None;
        let map = MapFile::new_from_map_str(&contents);
        self.all_entries = build_symbol_entries(&map);
        self.recompute_groups();
        self.drilldown_group = None;
        self.expanded_section = None;
        self.search_query.clear();
        cx.notify();
    }

    pub fn recompute_groups(&mut self) {
        self.aggregate = aggregate(&self.all_entries, self.show_debug);
    }

    pub fn select_page(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.current_page == page {
            return;
        }
        self.current_page = page;
        self.drilldown_group = None;
        self.expanded_section = None;
        self.search_query.clear();
        cx.notify();
    }

    pub fn toggle_debug(&mut self, cx: &mut Context<Self>) {
        self.show_debug = !self.show_debug;
        self.recompute_groups();
        cx.notify();
    }

    pub fn toggle_group_sort(&mut self, cx: &mut Context<Self>) {
        self.sort_ascending = !self.sort_ascending;
        cx.notify();
    }

    pub fn drill_into(&mut self, group: String, cx: &mut Context<Self>) {
        self.drilldown_group = Some(group);
        self.search_query.clear();
        cx.notify();
    }

    pub fn drill_out(&mut self, cx: &mut Context<Self>) {
        self.drilldown_group = None;
        self.search_query.clear();
        cx.notify();
    }

    pub fn toggle_section(&mut self, name: String, cx: &mut Context<Self>) {
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
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(toolbar::render(self, cx))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(pages::render(self, cx)),
            )
    }
}
