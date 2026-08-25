use crate::font::store::FontId;

pub mod cache;
pub mod command;
mod field;
pub mod inner;
pub mod writer;

pub const PREFIX_TEXT: &str = "raine1@Arctic~ %";
pub const CONSOLE_FONT: FontId = FontId::UBUNTU_MONO;
