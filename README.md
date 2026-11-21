# spritore

spritore turns SVG icons into the PNG sprite sheet and JSON index used by
[MapLibre GL](https://maplibre.org/) styles. It can also rasterize a single SVG
into RGBA pixels for `map.addImage` previews.

Use spritore from a command line, a Rust application, a browser, or Node. It is
well suited to style build pipelines, CI asset generation, map editors with
live icon previews, and applications that let users export sprite assets.

## Choose an interface

| Interface | Good for | Start with |
| --- | --- | --- |
| npm CLI | JavaScript projects and CI jobs | `npx @kartore/spritore build ...` |
| Cargo CLI | Rust-oriented build environments | `cargo install spritore` |
| Browser API | Map editors, previews, and in-browser exports | `@kartore/spritore` |
| Node API | Build scripts and server-side asset generation | `@kartore/spritore/node` |
| Rust API | Applications that process SVG and sprite data in memory | `spritore` or `spritore-core` |

## Install

For browser, Node, or npm CLI use:

```sh
pnpm add @kartore/spritore
```

Or with npm:

```sh
npm install @kartore/spritore
```

For the Rust API:

```sh
cargo add spritore --no-default-features
```

Use `spritore-core` directly when your integration only needs the in-memory
rendering and sprite-building types:

```sh
cargo add spritore-core
```

Install the command-line interface with Cargo:

```sh
cargo install spritore
```

## CLI

Build standard- and high-density sprite assets from the lowercase `.svg` files
in a directory:

```sh
npx @kartore/spritore build ./icons -o ./public/sprites
```

The default output is:

```text
public/sprites/
├── sprite.png
├── sprite.json
├── sprite@2x.png
└── sprite@2x.json
```

The complete command is:

```text
spritore build <svg-dir> -o <out-dir> [--name sprite] [--ratio 1,2] [--fast] [--sdf] [--skip-invalid]
```

- `--name <name>` changes the output basename.
- `--ratio <ratios>` accepts comma-separated integers from 1 to 255.
- `--fast` prioritizes generation speed over PNG file size.
- `--sdf` renders every icon as a MapLibre SDF icon (see [SDF icons](#sdf-icons)).
- `--skip-invalid` reports SVG parse errors and continues with valid icons.

Without `--skip-invalid`, an invalid SVG stops the command before output files
are written. Icon IDs come from filename stems; characters outside
`a-zA-Z0-9_-` become `-`, and collisions after conversion are errors.

The Cargo-installed command uses the same syntax:

```sh
spritore build ./icons -o ./public/sprites
```

## Browser

The rendering functions initialize the bundled WebAssembly module on their
first call. `renderIcon` returns the dimensions and straight-alpha RGBA data
expected by MapLibre's `map.addImage` API.

```ts
import {
	buildSpriteSheet,
	renderIcon,
} from "@kartore/spritore";

const markerSvg = `
	<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
		<circle cx="8" cy="8" r="6" fill="#4264fb" />
	</svg>
`;

const marker = await renderIcon("marker", markerSvg, 2);
map.addImage("marker", {
	width: marker.width,
	height: marker.height,
	data: marker.pixels,
});

const sprite = await buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	2,
);
```

`sprite.png` is a `Uint8Array`, `sprite.index` is the parsed MapLibre index,
and `sprite.indexJson` is a ready-to-write JSON string.

## Node

Import the `/node` entry point when working outside a browser:

```ts
import { readFile, writeFile } from "node:fs/promises";

import { buildSpriteSheet } from "@kartore/spritore/node";

const markerSvg = await readFile("marker.svg", "utf8");
const sprite = await buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	1,
	{ fast: true },
);

await writeFile("sprite.png", sprite.png);
await writeFile("sprite.json", sprite.indexJson);
```

See the [npm package README](js/README.md) for advanced WebAssembly input
options and the complete TypeScript API. Result objects and sprite index
entries are frozen plain objects; returned byte arrays are caller-owned and
need no cleanup.

## Rust

The Rust API accepts SVG strings and returns the PNG bytes and MapLibre index:

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

See the [`spritore` crate README](crates/spritore-cli/README.md) for Cargo
features and the [`spritore-core` README](crates/spritore-core/README.md) for
the lower-level API.

## SDF icons

MapLibre can tint single-colour icons with `icon-color` and draw
`icon-halo-*` around them when the sprite marks them as SDF (signed distance
field) icons. spritore renders an icon as an SDF when asked:

- the image keeps only the icon's alpha (RGB is zero), so original colours are lost;
- 3 logical pixels of transparent padding are added on every side (6 pixels at
  pixel ratio 2), so halos up to 3 pixels wide fit. The padding makes the image
  larger, so anchors such as `icon-anchor: bottom` move by that amount;
- distances use a radius of 8 logical pixels and a cutoff of 0.25, matching
  MapLibre's SDF shader;
- the sprite index entry gets `"sdf": true`.

In JavaScript, pass `sdf` per icon or per render:

```ts
await buildSpriteSheet([{ id: "pin", svg, sdf: true }], 2);
const pin = await renderIcon("pin", svg, 2, { sdf: true });
map.addImage("pin", pin, { pixelRatio: 2, sdf: pin.sdf });
```

In Rust, use `render_icon_with_options(id, svg, ratio, RenderOptions { sdf: true })`.
The CLI's `--sdf` applies to every icon.

## Stretchable icons (9-slice)

For `icon-text-fit`, MapLibre reads per-icon `stretchX`, `stretchY`,
`content`, `textFitWidth`, and `textFitHeight` from the sprite index. Pass
them with an icon in **logical pixels of the SVG**; spritore writes them in
image pixels for each pixel ratio (and shifted by the padding for SDF icons):

```ts
await buildSpriteSheet(
	[
		{
			id: "label-box",
			svg,
			stretchX: [[4, 20]],
			stretchY: [[4, 12]],
			content: [4, 4, 20, 12],
			textFitWidth: "stretchOrShrink",
		},
	],
	2,
);
// sprite@2x.json: "stretchX": [[8, 40]], "stretchY": [[8, 24]], "content": [8, 8, 40, 24], …

const box = await renderIcon("label-box", svg, 2, { stretchX: [[4, 20]] });
map.addImage("label-box", box, { pixelRatio: 2, stretchX: box.stretchX });
```

Values must stay inside the image, and stretch ranges must be ascending and
non-overlapping (MapLibre's rules); otherwise rendering fails. `textFitWidth`
and `textFitHeight` accept `stretchOrShrink`, `stretchOnly`, or
`proportional`. The CLI does not set per-icon metadata.

## Build modes

The default mode prioritizes smaller PNG files and is intended for final
assets. Set `fast: true` in the JavaScript or Rust API, or pass `--fast` to the
CLI, when quicker preview generation matters more than file size.

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

pnpm install
pnpm build
pnpm typecheck
pnpm test
pnpm bench
```

Building the JavaScript package also requires the exact `wasm-bindgen-cli`
version declared in `crates/spritore-wasm/Cargo.toml` and Binaryen's
`wasm-opt`. See [js/README.md](js/README.md#development) for setup details.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT License](LICENSE-MIT), at your option.
