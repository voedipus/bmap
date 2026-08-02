//! Top application toolbar with file open, navigation tabs, search, and debug toggle.

use xilem::masonry::layout::AsUnit;
use xilem::masonry::properties::Padding;
use xilem::style::Style as _;
use xilem::view::{
    CrossAxisAlignment, FlexExt as _, button, flex_row, label, sized_box, switch, text_button,
    text_input,
};
use xilem::{AnyWidgetView, Color, WidgetView};

use crate::app::{AppState, Page};
use crate::i18n::t;
use crate::theme::{ACCENT, BORDER, FG, TOOLBAR_BG};

pub fn render(data: &mut AppState) -> impl WidgetView<AppState> + use<> {
    let open_btn = text_button(t("open", data.language), |data: &mut AppState| {
        data.dialog_open = true;
    });

    let tab_specs = [
        (Page::Files, t("files", data.language)),
        (Page::Modules, t("modules", data.language)),
        (Page::Sections, t("sections", data.language)),
        (Page::Summary, t("summary", data.language)),
    ];
    let tabs: Vec<Box<AnyWidgetView<AppState>>> = tab_specs
        .into_iter()
        .map(|(page, label_text)| {
            let active = data.current_page == page;
            button(
                label(label_text).color(if active { TOOLBAR_BG } else { FG }),
                move |data: &mut AppState| data.select_page(page),
            )
            .padding(Padding::from_vh(4.0.px(), 12.0.px()))
            .corner_radius(6.0.px())
            .background_color(if active { ACCENT } else { Color::TRANSPARENT })
            .boxed()
        })
        .collect();

    let search = text_input(data.search_query.clone(), |data: &mut AppState, value| {
        data.search_query = value;
    })
    .placeholder(t("search-placeholder", data.language))
    .flex(1.0);

    let debug_toggle = flex_row((
        switch(data.show_debug, |data: &mut AppState, on| {
            data.show_debug = on;
            data.recompute_groups();
        }),
        label(t("label-debug", data.language)).text_size(13.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .gap(6.0.px());

    sized_box(
        flex_row((open_btn, tabs, search, debug_toggle))
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .gap(6.0.px()),
    )
    .background_color(TOOLBAR_BG)
    .border(BORDER, 1.0.px())
    .padding(Padding::from_vh(8.0.px(), 10.0.px()))
}
