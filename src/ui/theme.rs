//! Strict monochrome ANSI palette shared by every UI component.

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";

/// Bright white text.
pub const WHITE: &str = "\x1b[97m";
/// Light grey text.
pub const SILVER: &str = "\x1b[37m";
/// Dark grey text.
pub const GRAY: &str = "\x1b[90m";
/// White background, black bold text.
pub const INV_WHITE: &str = "\x1b[7;1;97m";
/// Light grey background, black text.
pub const INV_GRAY: &str = "\x1b[7;37m";
