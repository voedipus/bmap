//! Minimal focusable text input used for search queries.

use crate::theme;
use gpui::*;

#[derive(Clone, Debug)]
pub enum InputEvent {
    Changed(SharedString),
}

pub struct SearchInput {
    focus_handle: FocusHandle,
    text: SharedString,
    placeholder: SharedString,
}

impl EventEmitter<InputEvent> for SearchInput {}

impl SearchInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            text: "".into(),
            placeholder: placeholder.into(),
        }
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.text = text.into();
        cx.emit(InputEvent::Changed(self.text.clone()));
        cx.notify();
    }
}

impl Render for SearchInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        let display = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            self.text.clone()
        };
        let text_color = if self.text.is_empty() {
            theme::TEXT_SECONDARY
        } else {
            theme::TEXT_PRIMARY
        };

        div()
            .id("search-input")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_row()
            .flex_1()
            .h_8()
            .px_3()
            .py_1()
            .border_1()
            .rounded_md()
            .border_color(if is_focused {
                theme::ACCENT
            } else {
                theme::BORDER
            })
            .bg(theme::BG_TERTIARY)
            .text_color(text_color)
            .text_sm()
            .child(display)
            .on_click(cx.listener(|this, _event, window, cx| {
                this.focus_handle.focus(window, cx);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.to_ascii_lowercase();
                if key == "backspace" {
                    let mut text = this.text.to_string();
                    text.pop();
                    this.set_text(text, cx);
                } else if let Some(ch) = event
                    .keystroke
                    .key_char
                    .as_ref()
                    .and_then(|k| k.chars().next())
                    .filter(|c| !c.is_control())
                {
                    let mut text = this.text.to_string();
                    text.push(ch);
                    this.set_text(text, cx);
                }
            }))
    }
}
