use std::io::Cursor;

use spritore_core::{BuildOptions, RenderedIcon, SpriteIndexEntry, build_sprite_sheet};

fn icon(id: &str, width: u32, height: u32, pixels: Vec<u8>) -> RenderedIcon {
	RenderedIcon {
		id: id.to_owned(),
		width,
		height,
		pixels,
	}
}

fn decode_rgba(png_bytes: &[u8]) -> (u32, u32, png::ColorType, Vec<u8>) {
	let mut decoder = png::Decoder::new(Cursor::new(png_bytes));
	decoder.set_transformations(png::Transformations::EXPAND);
	let mut reader = decoder.read_info().unwrap();
	let encoded_color_type = reader.info().color_type;
	let mut pixels = vec![0; reader.output_buffer_size()];
	let frame = reader.next_frame(&mut pixels).unwrap();
	assert_eq!(frame.color_type, png::ColorType::Rgba);
	pixels.truncate(frame.buffer_size());
	(frame.width, frame.height, encoded_color_type, pixels)
}

fn compose_expected(
	width: u32,
	height: u32,
	icons: &[RenderedIcon],
	index: &std::collections::BTreeMap<String, SpriteIndexEntry>,
) -> Vec<u8> {
	let mut expected = vec![0; width as usize * height as usize * 4];
	for icon in icons {
		let entry = index[&icon.id];
		for row in 0..icon.height {
			let source_start = (row * icon.width * 4) as usize;
			let source_end = source_start + (icon.width * 4) as usize;
			let destination_start = (((entry.y + row) * width + entry.x) * 4) as usize;
			let destination_end = destination_start + (icon.width * 4) as usize;
			expected[destination_start..destination_end]
				.copy_from_slice(&icon.pixels[source_start..source_end]);
		}
	}
	expected
}

#[test]
fn indexed_png_round_trips_to_the_composed_rgba_pixels() {
	let icons = vec![
		icon("red", 2, 1, vec![255, 0, 0, 255, 255, 0, 0, 128]),
		icon("green", 1, 2, vec![0, 255, 0, 255, 0, 255, 0, 64]),
	];
	let sheet = build_sprite_sheet(&icons, 1, BuildOptions { fast: true }).unwrap();
	let (width, height, color_type, decoded) = decode_rgba(&sheet.png);
	assert_eq!(color_type, png::ColorType::Indexed);
	assert_eq!(
		decoded,
		compose_expected(width, height, &icons, &sheet.index)
	);
}

#[test]
fn rgba_png_round_trips_to_the_composed_rgba_pixels() {
	let pixels = (0..257)
		.flat_map(|value| [(value & 0xff) as u8, (value >> 8) as u8, 127, 255])
		.collect();
	let icons = vec![icon("many-colors", 257, 1, pixels)];
	let sheet = build_sprite_sheet(&icons, 1, BuildOptions { fast: true }).unwrap();
	let (width, height, color_type, decoded) = decode_rgba(&sheet.png);
	assert_eq!(color_type, png::ColorType::Rgba);
	assert_eq!(
		decoded,
		compose_expected(width, height, &icons, &sheet.index)
	);
}
