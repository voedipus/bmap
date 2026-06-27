//! Small helpers for rendering table headers and cells.

use crate::theme;
use gpui::prelude::*;
use gpui::*;

pub fn header_row() -> Div {
    div()
        .flex()
        .flex_row()
        .border_b_1()
        .border_color(theme::BORDER)
        .text_color(theme::TEXT_SECONDARY)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
}

pub fn header_row_cell(label: impl Into<SharedString>, width: Option<Pixels>) -> Div {
    let mut el = div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .py_1()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .child(label.into());
    if let Some(width) = width {
        el = el.w(width);
    } else {
        el = el.flex_1();
    }
    el
}

pub fn row() -> Div {
    div()
        .flex()
        .flex_row()
        .border_b_1()
        .border_color(theme::BORDER)
        .hover(|s| s.bg(theme::BG_HOVER))
}

pub fn text_cell(label: impl Into<SharedString>, width: Option<Pixels>) -> Div {
    let mut el = div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .py_1()
        .text_sm()
        .child(label.into());
    if let Some(width) = width {
        el = el.w(width);
    } else {
        el = el.flex_1();
    }
    el
}

pub fn numeric_cell(label: impl Into<SharedString>, width: Pixels) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .px_3()
        .py_1()
        .text_sm()
        .w(width)
        .child(label.into())
}
