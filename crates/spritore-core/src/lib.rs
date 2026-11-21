//! Rasterize SVG icons and build MapLibre-compatible sprite assets.
//!
//! Use [`render_icon`] to produce RGBA pixels for one icon, or combine rendered
//! icons with [`build_sprite_sheet`] and serialize its index with
//! [`index_to_json`]. The API accepts SVG text and returns in-memory image and
//! index data, so applications can choose how to read and store their assets.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod metadata;
mod png;
mod potpack;
mod sdf;

pub use metadata::{ImageMetadata, TextFit};

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// An error produced while rendering an icon or building a sprite sheet.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// The icon source could not be parsed as SVG.
	#[error("invalid SVG for icon `{id}`: {reason}")]
	InvalidSvg {
		/// The caller-provided icon identifier.
		id: String,
		/// The parser's description of the failure.
		reason: String,
	},
	/// More than one icon used the same identifier.
	#[error("duplicate icon id `{0}`")]
	DuplicateId(String),
	/// No icons were provided to the sprite builder.
	#[error("no icons given")]
	Empty,
	/// An icon had a zero pixel width or height.
	#[error("icon `{id}` has zero size")]
	ZeroSize {
		/// The caller-provided icon identifier.
		id: String,
	},
	/// An icon's stretch, content, or text-fit metadata did not fit its image.
	#[error("invalid metadata for icon `{id}`: {reason}")]
	InvalidMetadata {
		/// The caller-provided icon identifier.
		id: String,
		/// What is wrong with the metadata.
		reason: String,
	},
}

/// A rasterized icon with straight-alpha RGBA pixels.
pub struct RenderedIcon {
	/// The identifier used as the sprite index key.
	pub id: String,
	/// The pixel width after applying the requested pixel ratio.
	pub width: u32,
	/// The pixel height after applying the requested pixel ratio.
	pub height: u32,
	/// Straight (non-premultiplied) RGBA pixels in row-major order.
	///
	/// For SDF icons, RGB is zero and alpha holds the distance field.
	pub pixels: Vec<u8>,
	/// `true` when `pixels` hold a signed distance field for a MapLibre SDF icon.
	pub sdf: bool,
	/// Stretch, content, and text-fit metadata in image pixels (empty when unset).
	pub metadata: ImageMetadata,
}

/// Options for rendering one icon.
#[derive(Clone, Default, Debug)]
pub struct RenderOptions {
	/// Converts the icon to a signed distance field for MapLibre `sdf` icons.
	///
	/// Only the icon's alpha is kept, so MapLibre can tint it with `icon-color`
	/// and draw `icon-halo-*`. The image gains 3 logical pixels of transparent
	/// padding on every side (scaled by the pixel ratio), and the distance is
	/// encoded with a radius of 8 logical pixels and a cutoff of 0.25.
	pub sdf: bool,
	/// MapLibre `icon-text-fit` metadata in logical pixels of the SVG.
	///
	/// The rendered icon and its sprite index entry carry it in image pixels:
	/// multiplied by the pixel ratio and, for SDF icons, shifted by the padding.
	/// Values outside the image are an [`Error::InvalidMetadata`].
	pub metadata: ImageMetadata,
}

/// Rasterizes one SVG icon at the requested pixel ratio.
///
/// This is also suitable for producing individual images for MapLibre's
/// `map.addImage` API.
pub fn render_icon(id: &str, svg: &str, pixel_ratio: u8) -> Result<RenderedIcon, Error> {
	render_icon_with_options(id, svg, pixel_ratio, RenderOptions::default())
}

/// Rasterizes one SVG icon at the requested pixel ratio with [`RenderOptions`].
///
/// With `sdf` set, the result is a signed distance field that should be added
/// to MapLibre with `sdf: true`.
pub fn render_icon_with_options(
	id: &str,
	svg: &str,
	pixel_ratio: u8,
	render_options: RenderOptions,
) -> Result<RenderedIcon, Error> {
	let options = resvg::usvg::Options::default();
	let tree = resvg::usvg::Tree::from_str(svg, &options).map_err(|error| Error::InvalidSvg {
		id: id.to_owned(),
		reason: error.to_string(),
	})?;
	let size = tree.size();
	let ratio = f32::from(pixel_ratio);
	let width = (size.width() * ratio).ceil() as u32;
	let height = (size.height() * ratio).ceil() as u32;
	if width == 0 || height == 0 {
		return Err(Error::ZeroSize { id: id.to_owned() });
	}

	let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
		.ok_or_else(|| Error::ZeroSize { id: id.to_owned() })?;
	resvg::render(
		&tree,
		resvg::tiny_skia::Transform::from_scale(ratio, ratio),
		&mut pixmap.as_mut(),
	);

	// tiny-skia の内部表現は premultiplied RGBA なので、PNG と公開 API 用に戻す。
	let mut pixels = Vec::with_capacity(pixmap.data().len());
	for pixel in pixmap.pixels() {
		let straight = pixel.demultiply();
		pixels.extend_from_slice(&[
			straight.red(),
			straight.green(),
			straight.blue(),
			straight.alpha(),
		]);
	}

	let (width, height, pixels, offset) = if render_options.sdf {
		let (width, height, pixels) = sdf::alpha_to_sdf(width, height, &pixels, pixel_ratio);
		(width, height, pixels, f64::from(sdf::SDF_BUFFER))
	} else {
		(width, height, pixels, 0.0)
	};
	let metadata = render_options.metadata.to_image_pixels(pixel_ratio, offset);
	metadata
		.validate(width, height)
		.map_err(|reason| Error::InvalidMetadata {
			id: id.to_owned(),
			reason,
		})?;

	Ok(RenderedIcon {
		id: id.to_owned(),
		width,
		height,
		pixels,
		sdf: render_options.sdf,
		metadata,
	})
}

/// A MapLibre sprite index entry.
#[derive(Clone, PartialEq, Debug)]
pub struct SpriteIndexEntry {
	/// The icon's left coordinate in the sprite sheet.
	pub x: u32,
	/// The icon's top coordinate in the sprite sheet.
	pub y: u32,
	/// The icon's pixel width, excluding its packing margin.
	pub width: u32,
	/// The icon's pixel height, excluding its packing margin.
	pub height: u32,
	/// The pixel ratio represented by this sheet.
	pub pixel_ratio: u8,
	/// Marks the entry as a MapLibre SDF icon.
	pub sdf: bool,
	/// Stretch, content, and text-fit metadata in image pixels (empty when unset).
	pub metadata: ImageMetadata,
}

/// A PNG sprite sheet and its MapLibre index.
pub struct SpriteSheet {
	/// The complete encoded PNG file.
	pub png: Vec<u8>,
	/// Sprite entries keyed by icon identifier.
	pub index: BTreeMap<String, SpriteIndexEntry>,
}

/// Options controlling sprite sheet construction.
#[derive(Clone, Copy, Default)]
pub struct BuildOptions {
	/// Prioritizes generation speed over PNG file size when `true`.
	pub fast: bool,
}

struct Deduplicated<'a> {
	representatives: Vec<&'a RenderedIcon>,
	representative_for_icon: Vec<usize>,
}

fn deduplicate<'a>(icons: &[&'a RenderedIcon]) -> Deduplicated<'a> {
	let mut representatives: Vec<&RenderedIcon> = Vec::new();
	let mut representative_for_icon = Vec::with_capacity(icons.len());

	for icon in icons {
		let representative = representatives.iter().position(|candidate| {
			candidate.width == icon.width
				&& candidate.height == icon.height
				&& candidate.sdf == icon.sdf
				&& candidate.pixels == icon.pixels
		});
		let representative = representative.unwrap_or_else(|| {
			representatives.push(icon);
			representatives.len() - 1
		});
		representative_for_icon.push(representative);
	}

	Deduplicated {
		representatives,
		representative_for_icon,
	}
}

/// Builds a MapLibre PNG sprite sheet from rasterized icons.
///
/// Returns an error when the slice is empty, identifiers are duplicated, or an
/// icon has a zero width or height.
pub fn build_sprite_sheet(
	icons: &[RenderedIcon],
	pixel_ratio: u8,
	options: BuildOptions,
) -> Result<SpriteSheet, Error> {
	if icons.is_empty() {
		return Err(Error::Empty);
	}

	let mut ordered: Vec<&RenderedIcon> = icons.iter().collect();
	ordered.sort_by(|left, right| left.id.cmp(&right.id));
	for pair in ordered.windows(2) {
		if pair[0].id == pair[1].id {
			return Err(Error::DuplicateId(pair[0].id.clone()));
		}
	}
	for icon in &ordered {
		if icon.width == 0 || icon.height == 0 {
			return Err(Error::ZeroSize {
				id: icon.id.clone(),
			});
		}
	}

	let deduplicated = deduplicate(&ordered);
	let rectangles = deduplicated
		.representatives
		.iter()
		.enumerate()
		.map(|(index, icon)| potpack::Rectangle {
			index,
			id: &icon.id,
			width: u64::from(icon.width) + 2,
			height: u64::from(icon.height) + 2,
		})
		.collect();
	let layout = potpack::pack(rectangles);
	let mut rgba = vec![0; layout.width as usize * layout.height as usize * 4];

	for (representative, &(packed_x, packed_y)) in
		deduplicated.representatives.iter().zip(&layout.positions)
	{
		let x = packed_x + 1;
		let y = packed_y + 1;
		for row in 0..representative.height {
			let source_start = (row * representative.width * 4) as usize;
			let source_end = source_start + (representative.width * 4) as usize;
			let destination_start = (((y + row) * layout.width + x) * 4) as usize;
			let destination_end = destination_start + (representative.width * 4) as usize;
			rgba[destination_start..destination_end]
				.copy_from_slice(&representative.pixels[source_start..source_end]);
		}
	}

	let mut index = BTreeMap::new();
	for (icon, &representative) in ordered.iter().zip(&deduplicated.representative_for_icon) {
		let (packed_x, packed_y) = layout.positions[representative];
		index.insert(
			icon.id.clone(),
			SpriteIndexEntry {
				x: packed_x + 1,
				y: packed_y + 1,
				width: icon.width,
				height: icon.height,
				pixel_ratio,
				sdf: icon.sdf,
				metadata: icon.metadata.clone(),
			},
		);
	}

	let png = png::encode(layout.width, layout.height, &rgba, options.fast);
	Ok(SpriteSheet { png, index })
}

/// Serializes a MapLibre sprite index as pretty JSON.
///
/// The result uses two-space indentation, the MapLibre field names `x`, `y`,
/// `width`, `height`, and `pixelRatio`, and a trailing newline. SDF entries
/// also get `"sdf": true` after `pixelRatio`, followed by any `stretchX`,
/// `stretchY`, `content`, `textFitWidth`, and `textFitHeight` metadata.
pub fn index_to_json(index: &BTreeMap<String, SpriteIndexEntry>) -> String {
	if index.is_empty() {
		return "{}\n".to_owned();
	}

	let mut json = String::from("{\n");
	for (entry_number, (id, entry)) in index.iter().enumerate() {
		json.push_str("  ");
		push_json_string(&mut json, id);
		json.push_str(": {\n");
		let mut fields = vec![
			format!("\"x\": {}", entry.x),
			format!("\"y\": {}", entry.y),
			format!("\"width\": {}", entry.width),
			format!("\"height\": {}", entry.height),
			format!("\"pixelRatio\": {}", entry.pixel_ratio),
		];
		if entry.sdf {
			fields.push("\"sdf\": true".to_owned());
		}
		fields.extend(entry.metadata.json_fields());
		for (field_number, field) in fields.iter().enumerate() {
			json.push_str("    ");
			json.push_str(field);
			if field_number + 1 != fields.len() {
				json.push(',');
			}
			json.push('\n');
		}
		json.push_str("  }");
		if entry_number + 1 != index.len() {
			json.push(',');
		}
		json.push('\n');
	}
	json.push_str("}\n");
	json
}

fn push_json_string(output: &mut String, value: &str) {
	output.push('"');
	for character in value.chars() {
		match character {
			'"' => output.push_str("\\\""),
			'\\' => output.push_str("\\\\"),
			'\u{08}' => output.push_str("\\b"),
			'\u{0c}' => output.push_str("\\f"),
			'\n' => output.push_str("\\n"),
			'\r' => output.push_str("\\r"),
			'\t' => output.push_str("\\t"),
			'\u{00}'..='\u{1f}' => {
				write!(output, "\\u{:04x}", character as u32)
					.expect("writing to a String cannot fail");
			}
			_ => output.push(character),
		}
	}
	output.push('"');
}

#[cfg(test)]
mod tests {
	use super::*;

	fn icon(id: &str, width: u32, height: u32, pixels: Vec<u8>) -> RenderedIcon {
		RenderedIcon {
			id: id.to_owned(),
			width,
			height,
			pixels,
			sdf: false,
			metadata: ImageMetadata::default(),
		}
	}

	#[test]
	fn deduplicates_identical_pixels_using_the_first_id() {
		let icons = vec![
			icon("z-copy", 1, 1, vec![10, 20, 30, 255]),
			icon("a-original", 1, 1, vec![10, 20, 30, 255]),
		];
		let mut ordered: Vec<_> = icons.iter().collect();
		ordered.sort_by(|left, right| left.id.cmp(&right.id));
		let deduplicated = deduplicate(&ordered);
		assert!(std::ptr::eq(deduplicated.representatives[0], ordered[0]));

		let sheet = build_sprite_sheet(&icons, 1, BuildOptions { fast: true }).unwrap();
		assert_eq!(sheet.index["a-original"], sheet.index["z-copy"]);
		assert_eq!(u32::from_be_bytes(sheet.png[16..20].try_into().unwrap()), 3);
		assert_eq!(u32::from_be_bytes(sheet.png[20..24].try_into().unwrap()), 3);
	}

	#[test]
	fn does_not_merge_sdf_and_plain_icons_with_identical_pixels() {
		let mut sdf_icon = icon("b-sdf", 1, 1, vec![0, 0, 0, 255]);
		sdf_icon.sdf = true;
		let icons = vec![icon("a-plain", 1, 1, vec![0, 0, 0, 255]), sdf_icon];
		let sheet = build_sprite_sheet(&icons, 1, BuildOptions { fast: true }).unwrap();
		assert_ne!(
			(sheet.index["a-plain"].x, sheet.index["a-plain"].y),
			(sheet.index["b-sdf"].x, sheet.index["b-sdf"].y)
		);
		assert!(!sheet.index["a-plain"].sdf);
		assert!(sheet.index["b-sdf"].sdf);
	}

	#[test]
	fn renders_sdf_icons_with_padding() {
		let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;
		let plain = render_icon("square", svg, 2).unwrap();
		let sdf = render_icon_with_options(
			"square",
			svg,
			2,
			RenderOptions {
				sdf: true,
				..RenderOptions::default()
			},
		)
		.unwrap();
		assert!(!plain.sdf);
		assert!(sdf.sdf);
		assert_eq!((plain.width, plain.height), (20, 20));
		assert_eq!((sdf.width, sdf.height), (32, 32));
	}

	#[test]
	fn renders_metadata_in_image_pixels() {
		let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;
		let metadata = ImageMetadata {
			stretch_x: vec![[2.0, 8.0]],
			stretch_y: vec![[3.0, 7.0]],
			content: Some([2.0, 2.0, 8.0, 8.0]),
			text_fit_width: Some(TextFit::StretchOnly),
			text_fit_height: None,
		};
		let plain = render_icon_with_options(
			"box",
			svg,
			2,
			RenderOptions {
				sdf: false,
				metadata: metadata.clone(),
			},
		)
		.unwrap();
		assert_eq!(plain.metadata.stretch_x, vec![[4.0, 16.0]]);
		assert_eq!(plain.metadata.content, Some([4.0, 4.0, 16.0, 16.0]));

		// SDF は 3 論理 px の余白の分ずれる
		let sdf = render_icon_with_options(
			"box",
			svg,
			2,
			RenderOptions {
				sdf: true,
				metadata,
			},
		)
		.unwrap();
		assert_eq!(sdf.metadata.stretch_x, vec![[10.0, 22.0]]);
		assert_eq!(sdf.metadata.stretch_y, vec![[12.0, 20.0]]);
		assert_eq!(sdf.metadata.text_fit_width, Some(TextFit::StretchOnly));

		let sheet = build_sprite_sheet(&[plain], 2, BuildOptions { fast: true }).unwrap();
		assert_eq!(sheet.index["box"].metadata.stretch_y, vec![[6.0, 14.0]]);
	}

	#[test]
	fn rejects_metadata_outside_the_image() {
		let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"/>"#;
		let result = render_icon_with_options(
			"box",
			svg,
			1,
			RenderOptions {
				metadata: ImageMetadata {
					stretch_x: vec![[2.0, 12.0]],
					..ImageMetadata::default()
				},
				..RenderOptions::default()
			},
		);
		assert!(matches!(result, Err(Error::InvalidMetadata { id, .. }) if id == "box"));
	}

	#[test]
	fn reports_required_error_cases() {
		assert!(matches!(
			build_sprite_sheet(&[], 1, BuildOptions::default()),
			Err(Error::Empty)
		));

		let duplicate = vec![
			icon("same", 1, 1, vec![0; 4]),
			icon("same", 1, 1, vec![255; 4]),
		];
		assert!(matches!(
			build_sprite_sheet(&duplicate, 1, BuildOptions::default()),
			Err(Error::DuplicateId(id)) if id == "same"
		));

		let zero = vec![icon("zero", 0, 1, Vec::new())];
		assert!(matches!(
			build_sprite_sheet(&zero, 1, BuildOptions::default()),
			Err(Error::ZeroSize { id }) if id == "zero"
		));

		assert!(matches!(
			render_icon("bad", "not svg", 1),
			Err(Error::InvalidSvg { id, .. }) if id == "bad"
		));
		assert!(matches!(
			render_icon(
				"zero-ratio",
				r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#,
				0,
			),
			Err(Error::ZeroSize { id }) if id == "zero-ratio"
		));
	}

	#[test]
	fn index_json_format_and_escaping_are_fixed() {
		let index = BTreeMap::from([
			(
				"a\n\"".to_owned(),
				SpriteIndexEntry {
					x: 1,
					y: 2,
					width: 3,
					height: 4,
					pixel_ratio: 2,
					sdf: false,
					metadata: ImageMetadata::default(),
				},
			),
			(
				"b".to_owned(),
				SpriteIndexEntry {
					x: 5,
					y: 6,
					width: 7,
					height: 8,
					pixel_ratio: 2,
					sdf: true,
					metadata: ImageMetadata {
						stretch_x: vec![[1.0, 6.0]],
						content: Some([1.0, 1.0, 6.0, 7.0]),
						text_fit_width: Some(TextFit::Proportional),
						..ImageMetadata::default()
					},
				},
			),
		]);
		assert_eq!(
			index_to_json(&index),
			concat!(
				"{\n",
				"  \"a\\n\\\"\": {\n",
				"    \"x\": 1,\n",
				"    \"y\": 2,\n",
				"    \"width\": 3,\n",
				"    \"height\": 4,\n",
				"    \"pixelRatio\": 2\n",
				"  },\n",
				"  \"b\": {\n",
				"    \"x\": 5,\n",
				"    \"y\": 6,\n",
				"    \"width\": 7,\n",
				"    \"height\": 8,\n",
				"    \"pixelRatio\": 2,\n",
				"    \"sdf\": true,\n",
				"    \"stretchX\": [[1, 6]],\n",
				"    \"content\": [1, 1, 6, 7],\n",
				"    \"textFitWidth\": \"proportional\"\n",
				"  }\n",
				"}\n",
			),
		);
		assert_eq!(index_to_json(&BTreeMap::new()), "{}\n");
	}
}
