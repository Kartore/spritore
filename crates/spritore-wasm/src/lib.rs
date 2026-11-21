//! WebAssembly exports for SVG icon rendering and MapLibre sprite generation.
//!
//! JavaScript and TypeScript applications can access these functions through
//! the `@kartore/spritore` package's browser and Node entry points. Use
//! [`render_icon`] for `map.addImage` pixel data or [`build_sprite_sheet`] for a
//! complete PNG sprite and JSON index.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use js_sys::{Array, JSON, Object, Reflect, Uint8Array};
use serde::Deserialize;
use spritore_core::{BuildOptions, ImageMetadata, RenderOptions, RenderedIcon, TextFit};
use wasm_bindgen::prelude::*;

/// アイコンごとの MapLibre メタデータ (論理ピクセル)。JS の名前 (stretchX など) で受ける
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct MetadataInput {
	stretch_x: Option<Vec<[f64; 2]>>,
	stretch_y: Option<Vec<[f64; 2]>>,
	content: Option<[f64; 4]>,
	text_fit_width: Option<String>,
	text_fit_height: Option<String>,
}

impl MetadataInput {
	fn into_metadata(self) -> Result<ImageMetadata, JsError> {
		let text_fit = |value: Option<String>, name: &str| {
			value
				.map(|value| {
					TextFit::parse(&value).ok_or_else(|| {
						JsError::new(&format!(
							"invalid {name} `{value}`: expected stretchOrShrink, stretchOnly, or proportional"
						))
					})
				})
				.transpose()
		};
		Ok(ImageMetadata {
			stretch_x: self.stretch_x.unwrap_or_default(),
			stretch_y: self.stretch_y.unwrap_or_default(),
			content: self.content,
			text_fit_width: text_fit(self.text_fit_width, "textFitWidth")?,
			text_fit_height: text_fit(self.text_fit_height, "textFitHeight")?,
		})
	}
}

#[derive(Deserialize)]
struct IconSource {
	id: String,
	svg: String,
	#[serde(default)]
	sdf: bool,
	#[serde(flatten)]
	metadata: MetadataInput,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct BuildOptionsInput {
	fast: bool,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RenderOptionsInput {
	sdf: bool,
	#[serde(flatten)]
	metadata: MetadataInput,
}

/// Rasterizes one SVG icon and returns its dimensions and straight-alpha RGBA
/// pixels.
///
/// The returned object has `id`, `width`, `height`, `pixels`, and `sdf`
/// properties and can be used as MapLibre `map.addImage` image data. Pass
/// `{ sdf: true }` as `options` to render a signed distance field. `stretchX`,
/// `stretchY`, `content`, `textFitWidth`, and `textFitHeight` options are given
/// in logical pixels and returned in image pixels, ready for `map.addImage`.
#[wasm_bindgen(js_name = renderIcon)]
pub fn render_icon(
	id: &str,
	svg: &str,
	pixel_ratio: u8,
	options: Option<JsValue>,
) -> Result<JsValue, JsError> {
	let options: RenderOptionsInput = deserialize_options(options, "render")?;
	let icon = spritore_core::render_icon_with_options(
		id,
		svg,
		pixel_ratio,
		RenderOptions {
			sdf: options.sdf,
			metadata: options.metadata.into_metadata()?,
		},
	)
	.map_err(core_error)?;
	rendered_icon_to_js(&icon)
}

/// Builds a PNG sprite sheet and MapLibre index from SVG icon sources.
///
/// `icons` is an array of objects with `id`, `svg`, and optional `sdf`
/// properties. The returned
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
	let options: BuildOptionsInput = deserialize_options(options, "build")?;
	let rendered = sources
		.into_iter()
		.map(|source| {
			let metadata = source.metadata.into_metadata()?;
			spritore_core::render_icon_with_options(
				&source.id,
				&source.svg,
				pixel_ratio,
				RenderOptions {
					sdf: source.sdf,
					metadata,
				},
			)
			.map_err(core_error)
		})
		.collect::<Result<Vec<_>, _>>()?;
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

fn deserialize_options<T: Default + serde::de::DeserializeOwned>(
	options: Option<JsValue>,
	kind: &str,
) -> Result<T, JsError> {
	match options {
		None => Ok(T::default()),
		Some(value) if value.is_null() || value.is_undefined() => Ok(T::default()),
		Some(value) => serde_wasm_bindgen::from_value(value)
			.map_err(|error| JsError::new(&format!("invalid {kind} options: {error}"))),
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
	set_property(&result, "sdf", &JsValue::from_bool(icon.sdf))?;
	set_metadata(&result, &icon.metadata)?;
	Ok(result.into())
}

fn numbers_to_js(values: &[f64]) -> JsValue {
	values
		.iter()
		.map(|value| JsValue::from_f64(*value))
		.collect::<Array>()
		.into()
}

fn ranges_to_js(ranges: &[[f64; 2]]) -> JsValue {
	ranges
		.iter()
		.map(|range| numbers_to_js(range))
		.collect::<Array>()
		.into()
}

/// 画像ピクセルのメタデータを、設定のあるものだけ MapLibre の名前で載せる
fn set_metadata(object: &Object, metadata: &ImageMetadata) -> Result<(), JsError> {
	if !metadata.stretch_x.is_empty() {
		set_property(object, "stretchX", &ranges_to_js(&metadata.stretch_x))?;
	}
	if !metadata.stretch_y.is_empty() {
		set_property(object, "stretchY", &ranges_to_js(&metadata.stretch_y))?;
	}
	if let Some(content) = metadata.content {
		set_property(object, "content", &numbers_to_js(&content))?;
	}
	if let Some(fit) = metadata.text_fit_width {
		set_property(object, "textFitWidth", &JsValue::from_str(fit.as_str()))?;
	}
	if let Some(fit) = metadata.text_fit_height {
		set_property(object, "textFitHeight", &JsValue::from_str(fit.as_str()))?;
	}
	Ok(())
}

fn set_property(object: &Object, name: &str, value: &JsValue) -> Result<(), JsError> {
	Reflect::set(object.as_ref(), &JsValue::from_str(name), value)
		.map(|_| ())
		.map_err(|_| JsError::new(&format!("failed to set `{name}` on a result object")))
}

fn core_error(error: spritore_core::Error) -> JsError {
	JsError::new(&error.to_string())
}
