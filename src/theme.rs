//! Color constants for the dark theme.

use xilem::Color;

/// Window / page background.
pub const BG: Color = Color::from_rgb8(0x18, 0x18, 0x1b);
/// Toolbar and table-header background.
pub const TOOLBAR_BG: Color = Color::from_rgb8(0x27, 0x27, 0x2a);
/// Border color.
pub const BORDER: Color = Color::from_rgb8(0x3f, 0x3f, 0x46);
/// Accent color (active tab).
pub const ACCENT: Color = Color::from_rgb8(0x3b, 0x7e, 0xe4);
/// Primary text.
pub const FG: Color = Color::from_rgb8(0xf0, 0xf0, 0xea);
/// Muted text.
pub const MUTED: Color = Color::from_rgb8(0xa0, 0xa0, 0x9a);
/// Zebra stripe tint for odd rows.
pub const ROW_ALT: Color = Color::from_rgba8(0x3f, 0x3f, 0x46, 0x40);
