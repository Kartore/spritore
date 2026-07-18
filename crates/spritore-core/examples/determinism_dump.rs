//! Review tooling: dumps the golden pipeline outputs into a directory so the
//! same binary can be compared across targets (native vs wasm32-wasip1 via
//! `node:wasi`). Usage: `determinism_dump <out-dir>`.

use spritore_core::{BuildOptions, build_sprite_sheet, index_to_json, render_icon};

const FIXTURES: &[(&str, &str)] = &[
	("dot", include_str!("../tests/fixtures/dot.svg")),
	("pin", include_str!("../tests/fixtures/pin.svg")),
	("grad", include_str!("../tests/fixtures/grad.svg")),
	("line", include_str!("../tests/fixtures/line.svg")),
	("dot-copy", include_str!("../tests/fixtures/dot-copy.svg")),
	("spectrum", include_str!("../tests/fixtures/spectrum.svg")),
];

fn main() {
	let out_dir = std::env::args()
		.nth(1)
		.expect("usage: determinism_dump <out-dir>");
	std::fs::create_dir_all(&out_dir).expect("create out dir");

	for pixel_ratio in [1u8, 2] {
		let icons: Vec<_> = FIXTURES
			.iter()
			.map(|(id, svg)| render_icon(id, svg, pixel_ratio).expect("render"))
			.collect();
		let suffix = if pixel_ratio == 1 { "" } else { "@2x" };
		for fast in [true, false] {
			let sheet =
				build_sprite_sheet(&icons, pixel_ratio, BuildOptions { fast }).expect("build");
			let kind = if fast { ".fast" } else { "" };
			std::fs::write(format!("{out_dir}/sprite{suffix}{kind}.png"), &sheet.png)
				.expect("write png");
			if fast {
				std::fs::write(
					format!("{out_dir}/sprite{suffix}.json"),
					index_to_json(&sheet.index),
				)
				.expect("write json");
			}
		}
	}
	println!("done");
}
