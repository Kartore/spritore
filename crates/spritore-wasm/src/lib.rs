//! WebAssembly exports for SVG icon rendering and MapLibre sprite generation.
//!
//! JavaScript and TypeScript applications can access these functions through
//! the `@kartore/spritore` package's browser and Node entry points. Use
//! [`render_icon`] for `map.addImage` pixel data or [`build_sprite_sheet`] for a
//! complete PNG sprite and JSON index.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use js_sys::{JSON, Object, Reflect, Uint8Array};
use serde::Deserialize;
use spritore_core::{BuildOptions, RenderedIcon};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct IconSource {
	id: String,
	svg: String,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct BuildOptionsInput {
	fast: bool,
}

/// Rasterizes one SVG icon and returns its dimensions and straight-alpha RGBA
/// pixels.
///
/// The returned object has `id`, `width`, `height`, and `pixels` properties and
/// can be used as MapLibre `map.addImage` image data.
#[wasm_bindgen(js_name = renderIcon)]
pub fn render_icon(id: &str, svg: &str, pixel_ratio: u8) -> Result<JsValue, JsError> {
	let icon = spritore_core::render_icon(id, svg, pixel_ratio).map_err(core_error)?;
	rendered_icon_to_js(&icon)
}

/// Builds a PNG sprite sheet and MapLibre index from SVG icon sources.
///
/// `icons` is an array of objects with `id` and `svg` properties. The returned
/// object contains `png`, `index`, and ready-to-write `indexJson` properties.
/// Set the `fast` build option when generation speed matters more than PNG file
/// size.
#[wasm_bindgen(js_name = buildSpriteSheet)]
pub fn build_sprite_sheet(
	icons: JsValue,
	pixel_ratio: u8,
	options: Option<JsValue>,
) -> Result<JsValue, JsError> {
	let sources: Vec<IconSource> = serde_wasm_bindgen::from_value(icons)
		.map_err(|error| JsError::new(&format!("invalid icons: {error}")))?;
	let options = deserialize_options(options)?;
	let rendered = sources
		.iter()
		.map(|source| spritore_core::render_icon(&source.id, &source.svg, pixel_ratio))
		.collect::<Result<Vec<_>, _>>()
		.map_err(core_error)?;
	let sheet = spritore_core::build_sprite_sheet(
		&rendered,
		pixel_ratio,
		BuildOptions { fast: options.fast },
	)
	.map_err(core_error)?;

	let index_json = spritore_core::index_to_json(&sheet.index);
	let index = JSON::parse(&index_json)
		.map_err(|_| JsError::new("failed to parse the generated sprite index"))?;
	let result = Object::new();
	set_property(
		&result,
		"png",
		Uint8Array::from(sheet.png.as_slice()).as_ref(),
	)?;
	set_property(&result, "index", &index)?;
	set_property(&result, "indexJson", &JsValue::from_str(&index_json))?;
	Ok(result.into())
}

fn deserialize_options(options: Option<JsValue>) -> Result<BuildOptionsInput, JsError> {
	match options {
		None => Ok(BuildOptionsInput::default()),
		Some(value) if value.is_null() || value.is_undefined() => Ok(BuildOptionsInput::default()),
		Some(value) => serde_wasm_bindgen::from_value(value)
			.map_err(|error| JsError::new(&format!("invalid build options: {error}"))),
	}
}

fn rendered_icon_to_js(icon: &RenderedIcon) -> Result<JsValue, JsError> {
	let result = Object::new();
	set_property(&result, "id", &JsValue::from_str(&icon.id))?;
	set_property(&result, "width", &JsValue::from_f64(f64::from(icon.width)))?;
	set_property(
		&result,
		"height",
		&JsValue::from_f64(f64::from(icon.height)),
	)?;
	set_property(
		&result,
		"pixels",
		Uint8Array::from(icon.pixels.as_slice()).as_ref(),
	)?;
	Ok(result.into())
}

fn set_property(object: &Object, name: &str, value: &JsValue) -> Result<(), JsError> {
	Reflect::set(object.as_ref(), &JsValue::from_str(name), value)
		.map(|_| ())
		.map_err(|_| JsError::new(&format!("failed to set `{name}` on a result object")))
}

fn core_error(error: spritore_core::Error) -> JsError {
	JsError::new(&error.to_string())
}
