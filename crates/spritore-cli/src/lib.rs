//! MapLibre sprite generation for Rust.
//!
//! The default `cli` feature builds the `spritore` command-line interface.
//! Disable default features when only the Rust API is needed.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use spritore_core::{
	BuildOptions, Error, RenderedIcon, SpriteIndexEntry, SpriteSheet, build_sprite_sheet,
	index_to_json, render_icon,
};
