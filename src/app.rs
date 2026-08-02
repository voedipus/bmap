//! Application state and root view.

use std::path::PathBuf;

use mapfile_parser::MapFile;
use xilem::WidgetView;
use xilem::core::{MessageProxy, NoElement, ViewSequence, fork};
use xilem::view::{CrossAxisAlignment, FlexExt as _, flex_col, sized_box, task_raw};

use crate::i18n::{Language, t};
use crate::model::*;
use crate::ui::{empty, pages, toolbar};

/// Top-level pages shown in the toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Files,
    Modules,
    Sections,
    Summary,
}

/// Message sent from the background file-open task back to the app.
#[derive(Debug)]
pub enum FileMsg {
    Loaded {
        path: PathBuf,
        entries: Vec<MapEntry>,
    },
    Error {
        path: PathBuf,
        message: String,
    },
    Cancelled,
}

/// All application state, mutated by event handlers and read by [`app_logic`].
pub struct AppState {
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
    pub search_query: String,
    /// Indices into `all_entries` shown by the symbol drill-down list.
    pub symbol_rows: Vec<usize>,
    /// Set while the file dialog is open; gates the dialog [`task_raw`] view.
    pub dialog_open: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: Page::Files,
            language: Language::detect(),
            file_path: None,
            error: None,
            all_entries: Vec::new(),
            aggregate: Aggregate::default(),
            show_debug: false,
            sort_ascending: false,
            drilldown_group: None,
            expanded_section: None,
            search_query: String::new(),
            symbol_rows: Vec::new(),
            dialog_open: false,
        }
    }
}

impl AppState {
    pub fn recompute_groups(&mut self) {
        self.aggregate = aggregate(&self.all_entries, self.show_debug);
    }

    /// Rebuild the filtered and sorted drill-down row list from the current state.
    pub fn recompute_symbol_rows(&mut self) {
        let Some(group) = self.drilldown_group.clone() else {
            self.symbol_rows.clear();
            return;
        };
        let mut indices = drill_indices(&self.all_entries, &group, self.show_debug);
        if !self.search_query.is_empty() {
            let q = self.search_query.to_lowercase();
            indices.retain(|&i| {
                let e = &self.all_entries[i];
                e.name.to_lowercase().contains(&q)
                    || e.path_str.to_lowercase().contains(&q)
                    || e.section_type.to_lowercase().contains(&q)
            });
        }
        indices.sort_by_key(|&i| self.all_entries[i].size);
        if !self.sort_ascending {
            indices.reverse();
        }
        self.symbol_rows = indices;
    }

    pub fn select_page(&mut self, page: Page) {
        if self.current_page == page {
            return;
        }
        self.current_page = page;
        self.drilldown_group = None;
        self.expanded_section = None;
        self.search_query.clear();
    }

    pub fn drill_into(&mut self, group: String) {
        self.drilldown_group = Some(group);
        self.search_query.clear();
    }

    pub fn drill_out(&mut self) {
        self.drilldown_group = None;
        self.search_query.clear();
    }

    pub fn toggle_section(&mut self, name: String) {
        if self.expanded_section.as_ref() == Some(&name) {
            self.expanded_section = None;
        } else {
            self.expanded_section = Some(name);
        }
    }
}

/// Root component: recomposes the whole window whenever any state changes.
pub fn app_logic(data: &mut AppState) -> impl WidgetView<AppState> + use<> {
    // Keep the virtualized symbol list in sync with the current filters.
    if data.drilldown_group.is_some() {
        data.recompute_symbol_rows();
    }

    fork(
        flex_col((toolbar::render(data), sized_box(content(data)).flex(1.0)))
            .cross_axis_alignment(CrossAxisAlignment::Stretch),
        data.dialog_open.then(|| file_dialog_task(data.language)),
    )
}

fn content(data: &mut AppState) -> Box<xilem::AnyWidgetView<AppState>> {
    if data.all_entries.is_empty() {
        return empty::render(data);
    }
    match data.current_page {
        Page::Files | Page::Modules => {
            if data.drilldown_group.is_some() {
                pages::symbols_view(data)
            } else if data.current_page == Page::Files {
                pages::files_view(data)
            } else {
                pages::modules_view(data)
            }
        }
        Page::Sections => pages::sections_view(data),
        Page::Summary => pages::summary_view(data),
    }
}

/// Background task that shows the file dialog, reads and parses the chosen file
/// on a blocking thread, then sends the result back to the app.
///
/// Returns a `ViewSequence` (not a `WidgetView`): the task has no widget element
/// and is wired into the tree through `fork`'s alongside view.
fn file_dialog_task(
    language: Language,
) -> impl ViewSequence<AppState, (), xilem::ViewCtx, NoElement> + use<> {
    let title = t("open-map-file", language);
    task_raw(
        move |proxy: MessageProxy<FileMsg>, _state: &mut AppState| async move {
            let picked = rfd::AsyncFileDialog::new()
                .set_title(title)
                .add_filter("MAP files", &["map"])
                .add_filter("All files", &["*"])
                .pick_file()
                .await;
            let Some(handle) = picked else {
                let _ = proxy.message(FileMsg::Cancelled);
                return;
            };
            let path = handle.path().to_path_buf();
            let result =
                xilem::tokio::task::spawn_blocking(move || match std::fs::read_to_string(&path) {
                    Ok(text) => {
                        let map = MapFile::new_from_map_str(&text);
                        FileMsg::Loaded {
                            path,
                            entries: build_symbol_entries(&map),
                        }
                    }
                    Err(err) => FileMsg::Error {
                        path,
                        message: err.to_string(),
                    },
                })
                .await
                .expect("file read task should not panic");
            let _ = proxy.message(result);
        },
        |data: &mut AppState, msg| match msg {
            FileMsg::Loaded { path, entries } => {
                data.dialog_open = false;
                data.file_path = Some(path);
                data.error = None;
                data.all_entries = entries;
                data.recompute_groups();
                data.drilldown_group = None;
                data.expanded_section = None;
                data.search_query.clear();
                data.symbol_rows.clear();
            }
            FileMsg::Error { path, message } => {
                data.dialog_open = false;
                data.error = Some(format!("failed to read {}: {message}", path.display()));
            }
            FileMsg::Cancelled => {
                data.dialog_open = false;
            }
        },
    )
}
