use halcyon::mod_reexport;

use crate::font::store::FontId;

pub mod command;

mod_reexport!(cache);
mod_reexport!(writer);
mod_reexport!(inner);
mod_reexport!(field);

pub const PREFIX_TEXT: &str = "raine1@Arctic~ %";
pub const CONSOLE_FONT: FontId = FontId::UBUNTU_MONO;
