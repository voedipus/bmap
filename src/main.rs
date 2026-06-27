//! Entry point for `bmap`, a native GUI for inspecting linker MAP files.

mod app;
mod i18n;
mod model;
mod theme;
mod ui;

fn main() {
    app::run();
}
