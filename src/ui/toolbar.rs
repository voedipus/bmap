//! Top application toolbar with file open, navigation tabs, search, and debug toggle.

use crate::app::{BmapApp, Page};
use crate::i18n::t;
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::Input;
use gpui_component::switch::Switch;
use gpui_component::ActiveTheme;

pub fn render(app: &mut BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let open_btn = Button::new("open-btn")
        .label(t("open", app.language))
        .on_click(cx.listener(|app, _event, _window, cx| {
            app.open_file(cx);
        }));

    let active_bg = cx.theme().primary;
    let active_fg = cx.theme().background;
    let fg = cx.theme().foreground;
    let hover_bg = cx.theme().accent.opacity(0.15);

    let mut tab_divs = Vec::new();
    let tab_specs = [
        (Page::Files, t("files", app.language)),
        (Page::Modules, t("modules", app.language)),
        (Page::Sections, t("sections", app.language)),
        (Page::Summary, t("summary", app.language)),
    ];
    for (i, (page, label)) in tab_specs.into_iter().enumerate() {
        let active = app.current_page == page;
        tab_divs.push(
            div()
                .id(SharedString::from(format!("tab-{i}")))
                .px_3()
                .py_1()
                .rounded_md()
                .cursor_pointer()
                .text_sm()
                .text_color(if active { active_fg } else { fg })
                .bg(if active {
                    active_bg
                } else {
                    hsla(0., 0., 0., 0.)
                })
                .hover(move |s| s.bg(if active { active_bg } else { hover_bg }))
                .child(label)
                .on_click(cx.listener(move |app, _event, _window, cx| {
                    app.select_page(page, cx);
                })),
        );
    }

    let debug_label = t("label-debug", app.language);
    let debug_toggle = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_3()
        .cursor_pointer()
        .child(
            Switch::new("debug-toggle")
                .checked(app.show_debug)
                .on_click(cx.listener(|app, checked, _window, cx| {
                    if *checked != app.show_debug {
                        app.toggle_debug(cx);
                    }
                })),
        )
        .child(debug_label);

    let search = Input::new(&app.search_input).flex_1().appearance(true);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .p_2()
        .border_b_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().muted)
        .child(open_btn)
        .children(tab_divs)
        .child(search)
        .child(debug_toggle)
}
