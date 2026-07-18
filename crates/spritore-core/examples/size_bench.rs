//! Size benchmark over a directory of SVG icons (work order 1-7).
//! Usage: `size_bench <svg-dir> [out-dir]` — prints fast/zopfli sizes per
//! ratio and optionally writes the sheets for external comparison.

use std::time::Instant;

use spritore_core::{BuildOptions, build_sprite_sheet, index_to_json, render_icon};

fn main() {
	let svg_dir = std::env::args()
		.nth(1)
		.expect("usage: size_bench <svg-dir> [out-dir]");
	let out_dir = std::env::args().nth(2);

	let mut sources: Vec<(String, String)> = std::fs::read_dir(&svg_dir)
		.expect("read svg dir")
		.filter_map(|entry| {
			let path = entry.expect("dir entry").path();
			(path.extension().is_some_and(|extension| extension == "svg")).then(|| {
				let id = path
					.file_stem()
					.expect("stem")
					.to_string_lossy()
					.into_owned();
				(id, std::fs::read_to_string(&path).expect("read svg"))
			})
		})
		.collect();
	sources.sort_by(|left, right| left.0.cmp(&right.0));
	println!("{} icons from {svg_dir}", sources.len());

	for pixel_ratio in [1u8, 2] {
		let mut icons = Vec::new();
		let mut skipped = 0;
		for (id, svg) in &sources {
			match render_icon(id, svg, pixel_ratio) {
				Ok(icon) => icons.push(icon),
				Err(error) => {
					skipped += 1;
					eprintln!("skip {id}: {error}");
				}
			}
		}

		let t = Instant::now();
		let fast = build_sprite_sheet(&icons, pixel_ratio, BuildOptions { fast: true })
			.expect("build fast");
		let fast_ms = t.elapsed().as_millis();
		let t = Instant::now();
		let zopfli = build_sprite_sheet(&icons, pixel_ratio, BuildOptions { fast: false })
			.expect("build zopfli");
		let zopfli_ms = t.elapsed().as_millis();

		let color_type = fast.png[25];
		println!(
			"@{pixel_ratio}x: {} icons ({skipped} skipped) color_type={color_type} | fast {}B / {fast_ms}ms | zopfli {}B / {zopfli_ms}ms",
			icons.len(),
			fast.png.len(),
			zopfli.png.len(),
		);

		if let Some(out_dir) = &out_dir {
			std::fs::create_dir_all(out_dir).expect("create out dir");
			let suffix = if pixel_ratio == 1 { "" } else { "@2x" };
			std::fs::write(format!("{out_dir}/sprite{suffix}.png"), &zopfli.png).expect("write");
			std::fs::write(
				format!("{out_dir}/sprite{suffix}.json"),
				index_to_json(&zopfli.index),
			)
			.expect("write");
		}
	}
}
