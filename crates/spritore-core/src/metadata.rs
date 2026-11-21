//! MapLibre のアイコンのメタデータ (9-slice の stretchX / stretchY / content と textFit)。
//!
//! 入力は論理ピクセル (SVG のサイズ)。描画時にピクセル比と SDF の余白で画像ピクセルに直し、
//! MapLibre と同じ規則で検証する。

use std::fmt::Write as _;

/// Constraints on how `icon-text-fit` may scale an image on one axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextFit {
	/// Resize to tightly fit the content box to the text (MapLibre's default).
	StretchOrShrink,
	/// Resize to fit the text, but never below the content box's aspect ratio.
	StretchOnly,
	/// Resize to fit the text and keep the content box's aspect ratio.
	Proportional,
}

impl TextFit {
	/// The value MapLibre uses in a sprite index (`"stretchOrShrink"`, …).
	pub fn as_str(self) -> &'static str {
		match self {
			Self::StretchOrShrink => "stretchOrShrink",
			Self::StretchOnly => "stretchOnly",
			Self::Proportional => "proportional",
		}
	}

	/// Parses a sprite index value such as `"proportional"`.
	pub fn parse(value: &str) -> Option<Self> {
		match value {
			"stretchOrShrink" => Some(Self::StretchOrShrink),
			"stretchOnly" => Some(Self::StretchOnly),
			"proportional" => Some(Self::Proportional),
			_ => None,
		}
	}
}

/// MapLibre image metadata used by `icon-text-fit` (9-slice stretching).
///
/// As a render input the values are logical pixels of the SVG. Rendered icons
/// and sprite index entries hold them in image pixels: multiplied by the pixel
/// ratio and, for SDF icons, shifted by the SDF padding. Empty stretch lists
/// and `None` mean the field is absent.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct ImageMetadata {
	/// Horizontal ranges `[start, end]` that may be stretched, in ascending order.
	pub stretch_x: Vec<[f64; 2]>,
	/// Vertical ranges `[start, end]` that may be stretched, in ascending order.
	pub stretch_y: Vec<[f64; 2]>,
	/// The box `[left, top, right, bottom]` that text may cover.
	pub content: Option<[f64; 4]>,
	/// Constraints on horizontal scaling.
	pub text_fit_width: Option<TextFit>,
	/// Constraints on vertical scaling.
	pub text_fit_height: Option<TextFit>,
}

impl ImageMetadata {
	/// `true` when no field is set.
	pub fn is_empty(&self) -> bool {
		self.stretch_x.is_empty()
			&& self.stretch_y.is_empty()
			&& self.content.is_none()
			&& self.text_fit_width.is_none()
			&& self.text_fit_height.is_none()
	}

	/// 論理ピクセル → 画像ピクセル。`offset` は SDF の余白 (論理ピクセル)
	pub(crate) fn to_image_pixels(&self, pixel_ratio: u8, offset: f64) -> Self {
		let ratio = f64::from(pixel_ratio);
		let convert = |value: f64| (value + offset) * ratio;
		Self {
			stretch_x: self
				.stretch_x
				.iter()
				.map(|[start, end]| [convert(*start), convert(*end)])
				.collect(),
			stretch_y: self
				.stretch_y
				.iter()
				.map(|[start, end]| [convert(*start), convert(*end)])
				.collect(),
			content: self.content.map(|[left, top, right, bottom]| {
				[convert(left), convert(top), convert(right), convert(bottom)]
			}),
			text_fit_width: self.text_fit_width,
			text_fit_height: self.text_fit_height,
		}
	}

	/// MapLibre の ImageManager と同じ規則で画像ピクセルの値を検証する。問題があればその説明
	pub(crate) fn validate(&self, width: u32, height: u32) -> Result<(), String> {
		let all = self
			.stretch_x
			.iter()
			.chain(&self.stretch_y)
			.flatten()
			.chain(self.content.iter().flatten());
		if all.clone().any(|value| !value.is_finite()) {
			return Err("values must be finite numbers".to_owned());
		}
		validate_stretch("stretchX", &self.stretch_x, f64::from(width))?;
		validate_stretch("stretchY", &self.stretch_y, f64::from(height))?;
		if let Some([left, top, right, bottom]) = self.content {
			let (width, height) = (f64::from(width), f64::from(height));
			let inside = |value: f64, size: f64| (0.0..=size).contains(&value);
			if !inside(left, width)
				|| !inside(right, width)
				|| !inside(top, height)
				|| !inside(bottom, height)
				|| right < left
				|| bottom < top
			{
				return Err(
					"content must be a box [left, top, right, bottom] inside the image".to_owned(),
				);
			}
		}
		Ok(())
	}

	/// sprite index の JSON に足すフィールド (`"stretchX": [[a, b]]` など。値の無いものは出さない)
	pub(crate) fn json_fields(&self) -> Vec<String> {
		let mut fields = Vec::new();
		if !self.stretch_x.is_empty() {
			fields.push(format!("\"stretchX\": {}", ranges_json(&self.stretch_x)));
		}
		if !self.stretch_y.is_empty() {
			fields.push(format!("\"stretchY\": {}", ranges_json(&self.stretch_y)));
		}
		if let Some(content) = self.content {
			fields.push(format!("\"content\": {}", numbers_json(&content)));
		}
		if let Some(fit) = self.text_fit_width {
			fields.push(format!("\"textFitWidth\": \"{}\"", fit.as_str()));
		}
		if let Some(fit) = self.text_fit_height {
			fields.push(format!("\"textFitHeight\": \"{}\"", fit.as_str()));
		}
		fields
	}
}

fn validate_stretch(name: &str, ranges: &[[f64; 2]], size: f64) -> Result<(), String> {
	let mut last = 0.0;
	for &[start, end] in ranges {
		if start < last || end < start || size < end {
			return Err(format!(
				"{name} must be ascending, non-overlapping ranges [start, end] inside the image"
			));
		}
		last = end;
	}
	Ok(())
}

// f64 の Display は最短の往復表現で、全プラットフォームで同じ文字列になる (16.0 は "16")
fn number_json(value: f64) -> String {
	// -0 は "-0" になるので 0 に揃える
	if value == 0.0 {
		"0".to_owned()
	} else {
		value.to_string()
	}
}

fn numbers_json(values: &[f64]) -> String {
	let mut json = String::from("[");
	for (index, value) in values.iter().enumerate() {
		if index > 0 {
			json.push_str(", ");
		}
		json.push_str(&number_json(*value));
	}
	json.push(']');
	json
}

fn ranges_json(ranges: &[[f64; 2]]) -> String {
	let mut json = String::from("[");
	for (index, range) in ranges.iter().enumerate() {
		if index > 0 {
			json.push_str(", ");
		}
		write!(json, "{}", numbers_json(range)).expect("writing to a String cannot fail");
	}
	json.push(']');
	json
}

#[cfg(test)]
mod tests {
	use super::*;

	fn metadata() -> ImageMetadata {
		ImageMetadata {
			stretch_x: vec![[2.0, 4.0], [6.0, 8.0]],
			stretch_y: vec![[2.5, 7.5]],
			content: Some([2.0, 2.0, 8.0, 8.0]),
			text_fit_width: Some(TextFit::StretchOnly),
			text_fit_height: Some(TextFit::Proportional),
		}
	}

	#[test]
	fn converts_logical_pixels_with_ratio_and_offset() {
		let converted = metadata().to_image_pixels(2, 3.0);
		assert_eq!(converted.stretch_x, vec![[10.0, 14.0], [18.0, 22.0]]);
		assert_eq!(converted.stretch_y, vec![[11.0, 21.0]]);
		assert_eq!(converted.content, Some([10.0, 10.0, 22.0, 22.0]));
		assert_eq!(converted.text_fit_width, Some(TextFit::StretchOnly));
	}

	#[test]
	fn validates_like_maplibre() {
		assert!(metadata().validate(10, 10).is_ok());
		let mut overlapping = metadata();
		overlapping.stretch_x = vec![[2.0, 6.0], [4.0, 8.0]];
		assert!(
			overlapping
				.validate(10, 10)
				.unwrap_err()
				.contains("stretchX")
		);
		let mut outside = metadata();
		outside.stretch_y = vec![[2.0, 11.0]];
		assert!(outside.validate(10, 10).unwrap_err().contains("stretchY"));
		let mut reversed = metadata();
		reversed.content = Some([8.0, 2.0, 2.0, 8.0]);
		assert!(reversed.validate(10, 10).unwrap_err().contains("content"));
		let mut negative = metadata();
		negative.stretch_x = vec![[-1.0, 2.0]];
		assert!(negative.validate(10, 10).is_err());
		let mut infinite = metadata();
		infinite.content = Some([0.0, 0.0, f64::INFINITY, 1.0]);
		assert!(infinite.validate(10, 10).unwrap_err().contains("finite"));
	}

	#[test]
	fn json_fields_use_maplibre_names_and_short_numbers() {
		assert_eq!(
			metadata().json_fields(),
			vec![
				"\"stretchX\": [[2, 4], [6, 8]]",
				"\"stretchY\": [[2.5, 7.5]]",
				"\"content\": [2, 2, 8, 8]",
				"\"textFitWidth\": \"stretchOnly\"",
				"\"textFitHeight\": \"proportional\"",
			]
		);
		assert!(ImageMetadata::default().json_fields().is_empty());
		assert_eq!(number_json(-0.0), "0");
	}

	#[test]
	fn text_fit_round_trips() {
		for fit in [
			TextFit::StretchOrShrink,
			TextFit::StretchOnly,
			TextFit::Proportional,
		] {
			assert_eq!(TextFit::parse(fit.as_str()), Some(fit));
		}
		assert_eq!(TextFit::parse("stretch"), None);
	}
}
