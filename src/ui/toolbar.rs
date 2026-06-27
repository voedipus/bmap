//! Top application toolbar with file open, navigation, search, and debug toggle.

use crate::app::{BmapApp, Page};
use crate::i18n::t;
use crate::theme;
use gpui::prelude::*;
use gpui::*;

pub fn render(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let open_btn = styled_button(t("open", app.language))
        .id("open-btn")
        .on_click(cx.listener(|app, _event, _window, cx| app.open_file(cx)));

    let tabs = [
        (Page::Files, t("files", app.language)),
        (Page::Modules, t("modules", app.language)),
        (Page::Sections, t("sections", app.language)),
        (Page::Summary, t("summary", app.language)),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (page, label))| {
        let active = app.current_page == page;
        let bg = if active {
            theme::ACCENT
        } else {
            theme::BG_TERTIARY
        };
        let text = if active {
            theme::BG_PRIMARY
        } else {
            theme::TEXT_PRIMARY
        };
        div()
            .id(format!("tab-{i}"))
            .flex()
            .flex_row()
            .items_center()
            .px_3()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .bg(bg)
            .text_color(text)
            .text_sm()
            .hover(|s| {
                s.bg(if active {
                    theme::ACCENT
                } else {
                    theme::BG_HOVER
                })
            })
            .child(label)
            .on_click(cx.listener(move |app, _event, _window, cx| {
                app.select_page(page, cx);
            }))
    });

    let debug_label = format!(
        "[{}] {}",
        if app.show_debug { "x" } else { " " },
        t("label-debug", app.language)
    );
    let debug_btn = styled_button(debug_label)
        .id("debug-toggle")
        .on_click(cx.listener(|app, _event, _window, cx| app.toggle_debug(cx)));

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .p_2()
        .border_b_1()
        .border_color(theme::BORDER)
        .bg(theme::BG_SECONDARY)
        .child(open_btn)
        .children(tabs)
        .child(app.search_input.clone())
        .child(debug_btn)
}

fn styled_button(label: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .bg(theme::BG_TERTIARY)
        .text_color(theme::TEXT_PRIMARY)
        .text_sm()
        .hover(|s| s.bg(theme::BG_HOVER))
        .child(label.into())
}
