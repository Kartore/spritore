//! Golden files can be regenerated after an intentional format change with:
//! `cargo test -p spritore-core --test golden -- --ignored regen_golden`

use std::path::{Path, PathBuf};

use spritore_core::{
	BuildOptions, RenderedIcon, SpriteSheet, build_sprite_sheet, index_to_json, render_icon,
};

const FIXTURES: &[(&str, &str)] = &[
	("dot", include_str!("fixtures/dot.svg")),
	("pin", include_str!("fixtures/pin.svg")),
	("grad", include_str!("fixtures/grad.svg")),
	("line", include_str!("fixtures/line.svg")),
	("dot-copy", include_str!("fixtures/dot-copy.svg")),
	("spectrum", include_str!("fixtures/spectrum.svg")),
];

fn render_fixtures(pixel_ratio: u8, order: &[usize]) -> Vec<RenderedIcon> {
	order
		.iter()
		.map(|&index| {
			let (id, svg) = FIXTURES[index];
			render_icon(id, svg, pixel_ratio).unwrap()
		})
		.collect()
}

fn build_both(pixel_ratio: u8) -> (SpriteSheet, SpriteSheet) {
	let icons = render_fixtures(pixel_ratio, &[0, 1, 2, 3, 4, 5]);
	let fast = build_sprite_sheet(&icons, pixel_ratio, BuildOptions { fast: true }).unwrap();
	let zopfli = build_sprite_sheet(&icons, pixel_ratio, BuildOptions { fast: false }).unwrap();
	(fast, zopfli)
}

fn golden_dir() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn assert_golden(name: &str, actual: &[u8]) {
	let path = golden_dir().join(name);
	let expected = std::fs::read(&path)
		.unwrap_or_else(|error| panic!("failed to read golden file {}: {error}", path.display()));
	assert_eq!(actual, expected, "golden mismatch for {}", path.display());
}

#[test]
fn generated_assets_match_golden_bytes() {
	for pixel_ratio in [1, 2] {
		let (fast, zopfli) = build_both(pixel_ratio);
		assert_eq!(fast.index, zopfli.index);
		assert_eq!(fast.index["dot"], fast.index["dot-copy"]);
		// The spectrum fixture forces the complete sheet above the indexed-color threshold.
		assert_eq!(fast.png[25], 6);
		assert_eq!(zopfli.png[25], 6);

		let suffix = if pixel_ratio == 1 { "" } else { "@2x" };
		assert_golden(&format!("sprite{suffix}.fast.png"), &fast.png);
		assert_golden(&format!("sprite{suffix}.png"), &zopfli.png);
		assert_golden(
			&format!("sprite{suffix}.json"),
			index_to_json(&fast.index).as_bytes(),
		);
	}
}

#[test]
fn output_is_stable_across_repeated_and_permuted_inputs() {
	let canonical = render_fixtures(1, &[0, 1, 2, 3, 4, 5]);
	let first = build_sprite_sheet(&canonical, 1, BuildOptions { fast: true }).unwrap();
	let second = build_sprite_sheet(&canonical, 1, BuildOptions { fast: true }).unwrap();
	assert_eq!(first.png, second.png);
	assert_eq!(first.index, second.index);

	for order in [[5, 4, 3, 2, 1, 0], [2, 0, 5, 1, 4, 3], [4, 1, 3, 0, 2, 5]] {
		let permuted = render_fixtures(1, &order);
		let sheet = build_sprite_sheet(&permuted, 1, BuildOptions { fast: true }).unwrap();
		assert_eq!(
			first.png, sheet.png,
			"PNG changed for input order {order:?}"
		);
		assert_eq!(
			first.index, sheet.index,
			"index changed for input order {order:?}"
		);
	}
}

#[test]
#[ignore = "overwrites committed golden files intentionally"]
fn regen_golden() {
	let directory = golden_dir();
	std::fs::create_dir_all(&directory).unwrap();
	for pixel_ratio in [1, 2] {
		let (fast, zopfli) = build_both(pixel_ratio);
		let suffix = if pixel_ratio == 1 { "" } else { "@2x" };
		std::fs::write(directory.join(format!("sprite{suffix}.fast.png")), fast.png).unwrap();
		std::fs::write(directory.join(format!("sprite{suffix}.png")), zopfli.png).unwrap();
		std::fs::write(
			directory.join(format!("sprite{suffix}.json")),
			index_to_json(&fast.index),
		)
		.unwrap();
	}
}
