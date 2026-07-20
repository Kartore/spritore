//! Generate MapLibre-compatible sprite assets from Rust.
//!
//! [`render_icon`] returns RGBA pixels for one SVG, while
//! [`build_sprite_sheet`] produces a PNG sprite sheet and index. The default
//! `cli` feature also builds the `spritore` command-line interface; disable
//! default features when only the library API is needed. The command accepts a
//! directory of SVG files and writes the PNG and JSON files used by a MapLibre
//! style.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use spritore_core::{
	BuildOptions, Error, RenderedIcon, SpriteIndexEntry, SpriteSheet, build_sprite_sheet,
	index_to_json, render_icon,
};
