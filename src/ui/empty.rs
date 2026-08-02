//! Empty state shown before a file is loaded or when a view has no matches.

use xilem::masonry::layout::AsUnit;
use xilem::style::Style as _;
use xilem::view::{CrossAxisAlignment, FlexSpacer, flex_col, label};
use xilem::{AnyWidgetView, FontWeight, WidgetView};

use crate::app::AppState;
use crate::i18n::t;
use crate::theme::MUTED;

pub fn render(data: &AppState) -> Box<AnyWidgetView<AppState>> {
    let (icon, title, subtitle): (&str, &'static str, Option<&str>) = if let Some(err) = &data.error
    {
        (
            "\u{26a0}",
            t("error-loading", data.language),
            Some(err.as_str()),
        )
    } else {
        (
            "\u{1f4e6}",
            t("no-file-loaded", data.language),
            Some(t("open-instruction", data.language)),
        )
    };

    let mut content: Vec<Box<AnyWidgetView<AppState>>> = vec![
        label(icon).text_size(24.0).boxed(),
        label(title)
            .text_size(18.0)
            .weight(FontWeight::BOLD)
            .boxed(),
    ];
    if let Some(sub) = subtitle {
        content.push(label(sub).text_size(13.0).color(MUTED).boxed());
    }

    flex_col((
        FlexSpacer::Flex(1.0),
        flex_col(content).gap(8.0.px()),
        FlexSpacer::Flex(1.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .boxed()
}
