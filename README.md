# spritore

[MapLibre GL](https://maplibre.org/) sprite generation in Rust and WebAssembly.

spritore turns a set of SVG icons into a MapLibre-compatible PNG sprite sheet
and JSON index. Its API is available in Rust, browsers, and Node for both
complete sprite sheets and individual icon rasterization.

## Why spritore?

- **Browser and Node APIs** — the Rust core is exposed through one WebAssembly
  module, including a per-icon API for live map previews.
- **Rust library and native CLI** — use the same sprite engine from application
  code or install the `spritore` command.
- **Compact PNG output** — palette reduction, PNG filter selection, Zopfli
  compression, and pixel-identical icon deduplication are built in.

The npm package includes browser and Node APIs plus a command-line interface.
An equivalent native Rust CLI is also available from this repository.

## Install

### JavaScript

```sh
pnpm add @kartore/spritore
```

Or with npm:

```sh
npm install @kartore/spritore
```

### Rust

Disable default features when only the library API is needed:

```sh
cargo add spritore --no-default-features
```

## CLI

Build the default `sprite.png`, `sprite.json`, `sprite@2x.png`, and
`sprite@2x.json` files from a directory of SVG icons:

```sh
npx @kartore/spritore build ./icons -o ./public/sprites
```

The complete command is:

```text
spritore build <svg-dir> -o <out-dir> [--name sprite] [--ratio 1,2] [--fast] [--skip-invalid]
```

- `--name` changes the output basename.
- `--ratio` selects one or more comma-separated pixel ratios.
- `--fast` uses faster miniz compression instead of Zopfli.
- `--skip-invalid` reports and excludes SVG parse errors.

The native Rust CLI provides the same command:

```sh
cargo install spritore
spritore build ./icons -o ./public/sprites
```

## Quick start

### JavaScript

```js
import {
	buildSpriteSheet,
	init,
	renderIcon,
} from "@kartore/spritore";

const markerSvg = `
	<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
		<circle cx="8" cy="8" r="6" fill="#4264fb" />
	</svg>
`;

await init();

const marker = renderIcon("marker", markerSvg, 2);
const sprite = buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	2,
);

console.log(marker.width, marker.height, marker.pixels);
console.log(sprite.png, sprite.index, sprite.indexJson);
```

In Node, import the Node entry point. It reads the bundled wasm file from the
package and exposes the same API:

```js
import { readFile } from "node:fs/promises";

import { buildSpriteSheet, init } from "@kartore/spritore/node";

await init();
const markerSvg = await readFile("marker.svg", "utf8");
const sprite = buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	1,
	{ fast: true },
);
```

See the [package README](js/README.md) for the complete API and usage notes.

### Rust

```rust
use spritore::{BuildOptions, build_sprite_sheet, index_to_json, render_icon};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
		<circle cx="8" cy="8" r="6" fill="#4264fb" />
	</svg>"##;
	let icons = [render_icon("marker", svg, 1)?];
	let sheet = build_sprite_sheet(&icons, 1, BuildOptions::default())?;

	std::fs::write("sprite.png", sheet.png)?;
	std::fs::write("sprite.json", index_to_json(&sheet.index))?;
	Ok(())
}
```

See the [`spritore` crate README](crates/spritore-cli/README.md) for feature
selection and the low-level [`spritore-core` README](crates/spritore-core/README.md)
for direct core usage.

## Compression

The default compression mode uses Zopfli for smaller downloads. Pass
`{ fast: true }` to use miniz for faster previews. The two modes produce
different PNG encodings. Pixel-identical icons share one rectangle while
retaining separate index entries.

## SVG limitations

- External resources such as linked images and web fonts are not loaded.
- SVG `<text>` is not supported in this release because fonts are not bundled.
  Text elements may parse successfully but are not rendered.

## Development

The repository uses its pinned Rust toolchain plus Node and pnpm.

```sh
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace

pnpm -C js install
pnpm -C js build
pnpm -C js test
```

Building the JavaScript package also requires the exact `wasm-bindgen-cli`
version declared in `crates/spritore-wasm/Cargo.toml` and Binaryen's
`wasm-opt`. See [js/README.md](js/README.md#development) for setup details.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT License](LICENSE-MIT), at your option.
