//! Entry point for `bmap`, a native GUI for inspecting linker MAP files.

mod app;
mod i18n;
mod model;
mod theme;
mod ui;

use xilem::dpi::LogicalSize;
use xilem::{EventLoop, WindowOptions, Xilem};

use crate::app::{AppState, app_logic};

fn main() -> Result<(), xilem::winit::error::EventLoopError> {
    let window_options =
        WindowOptions::new("bmap").with_initial_inner_size(LogicalSize::new(960.0, 640.0));
    let app = Xilem::new_simple(AppState::default(), app_logic, window_options);
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
