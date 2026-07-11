//! Empty state shown before a file is loaded or when a view has no matches.

use crate::app::BmapApp;
use crate::i18n::t;
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;

pub fn render(app: &BmapApp, cx: &mut Context<BmapApp>) -> impl IntoElement {
    let (icon, title, subtitle): (&str, SharedString, Option<SharedString>) =
        if let Some(err) = &app.error {
            (
                "\u{26a0}",
                t("error-loading", app.language),
                Some(err.clone().into()),
            )
        } else {
            (
                "\u{1f4e6}",
                t("no-file-loaded", app.language),
                Some(t("open-instruction", app.language)),
            )
        };

    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap_4()
        .child(div().text_2xl().child(icon))
        .child(div().text_xl().child(title))
        .when_some(subtitle, |this, s| {
            this.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(s),
            )
        })
}
