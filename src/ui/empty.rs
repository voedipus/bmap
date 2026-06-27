//! Empty state shown before a file is loaded or when a view has no matches.

use crate::app::BmapApp;
use crate::i18n::t;
use crate::theme;
use gpui::prelude::*;
use gpui::*;

pub fn render(app: &BmapApp) -> impl IntoElement {
    let (icon, title, subtitle): (&str, SharedString, Option<SharedString>) =
        if let Some(err) = &app.error {
            (
                "⚠",
                t("error-loading", app.language),
                Some(err.clone().into()),
            )
        } else {
            (
                "📦",
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
            this.child(div().text_sm().text_color(theme::TEXT_SECONDARY).child(s))
        })
}
