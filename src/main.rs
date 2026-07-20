//! Entry point for `bmap`, a native GUI for inspecting linker MAP files.

mod app;
mod i18n;
mod model;
mod ui;

use gpui::AppContext as _;
use gpui_component::ActiveTheme;
use gpui_component::Root;
use gpui_component::Theme;
use gpui_component::ThemeMode;

use crate::app::BmapApp;

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_component::init(cx);

        cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                    origin: gpui::Point {
                        x: gpui::px(100.0),
                        y: gpui::px(100.0),
                    },
                    size: gpui::Size {
                        width: gpui::px(960.0),
                        height: gpui::px(640.0),
                    },
                })),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("bmap".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                // Force dark theme — sync_system_appearance may return Light on some Linux compositors
                let _ = Theme::sync_system_appearance(Some(window), cx);
                if !cx.theme().is_dark() {
                    Theme::change(ThemeMode::Dark, Some(window), cx);
                }

                let view = cx.new(|cx| BmapApp::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
