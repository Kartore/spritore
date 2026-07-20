# @kartore/spritore

`@kartore/spritore` turns SVG icons into the PNG sprite sheet and JSON index
used by [MapLibre GL](https://maplibre.org/) styles. It can also rasterize one
SVG into RGBA pixels for `map.addImage` previews.

The package can be used as a command, in a browser, or in Node. Typical uses
include generating style assets during a build, adding live icon previews to a
map editor, and letting users download a completed sprite from a web app.

## Install

```sh
pnpm add @kartore/spritore
```

Or with npm:

```sh
npm install @kartore/spritore
```

## CLI

The package includes a `spritore` executable. It reads lowercase `.svg` files
from a directory and writes MapLibre PNG and JSON sprite assets:

```sh
npx @kartore/spritore build ./icons -o ./public/sprites
```

By default it generates:

```text
public/sprites/
├── sprite.png
├── sprite.json
├── sprite@2x.png
└── sprite@2x.json
```

Available options:

```text
spritore build <svg-dir> -o <out-dir> [--name sprite] [--ratio 1,2] [--fast] [--skip-invalid]
```

- `--name <name>` changes the output basename.
- `--ratio <ratios>` accepts comma-separated integers from 1 to 255.
- `--fast` prioritizes generation speed over PNG file size.
- `--skip-invalid` reports SVG parse errors and continues with valid icons.

Without `--skip-invalid`, an invalid SVG stops the command before output files
are written. Icon IDs come from filename stems; characters outside
`a-zA-Z0-9_-` become `-`, and collisions after conversion are errors.

## Browser

The rendering functions initialize the bundled WebAssembly module
automatically on their first call:

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

Initialization is cached across both functions. Advanced integrations can use
the `wasm` option on the first call to provide custom WebAssembly bytes or a
URL; normal browser usage does not need this option.

## Node

Use the `/node` entry point so the packaged WebAssembly file is loaded from the
filesystem:

```ts
import { readFile, writeFile } from "node:fs/promises";

import { buildSpriteSheet } from "@kartore/spritore/node";

const markerSvg = await readFile("marker.svg", "utf8");
const sprite = await buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	1,
);

await writeFile("sprite.png", sprite.png);
await writeFile("sprite.json", sprite.indexJson);
```

Write `indexJson` directly when creating a sprite index. The `index` property
contains the parsed object for applications that need to inspect or transform
entries. Result objects, the index, and each index entry are frozen plain
objects. The `Uint8Array` values are caller-owned and require no manual
cleanup.

## API

### `renderIcon(id, svg, pixelRatio, options?)`

Rasterizes one SVG and asynchronously returns straight-alpha RGBA pixels:

```ts
type RenderedIcon = {
	readonly id: string;
	readonly width: number;
	readonly height: number;
	readonly pixels: Uint8Array;
};

type RenderIconOptions = {
	readonly wasm?: string | URL | ArrayBuffer | Uint8Array |
		Promise<string | URL | ArrayBuffer | Uint8Array>;
};
```

Use the returned `width`, `height`, and `pixels` as MapLibre's
`map.addImage` image data. A pixel ratio of `1` or `2` covers typical
standard- and high-density previews.

### `buildSpriteSheet(icons, pixelRatio, options?)`

Rasterizes a collection of SVG icons and asynchronously returns complete
MapLibre sprite assets:

```ts
type SpriteSheet = {
	readonly png: Uint8Array;
	readonly index: Readonly<Record<string, {
		readonly x: number;
		readonly y: number;
		readonly width: number;
		readonly height: number;
		readonly pixelRatio: number;
	}>>;
	readonly indexJson: string;
};

type BuildOptions = {
	readonly fast?: boolean;
	readonly wasm?: string | URL | ArrayBuffer | Uint8Array |
		Promise<string | URL | ArrayBuffer | Uint8Array>;
};
```

Each input item has an `id` used as its index key and an `svg` string. Duplicate
IDs, empty icon arrays, invalid SVG, and zero-sized icons are errors.

## Build modes

The default mode prioritizes smaller PNG files and is intended for final
assets. Pass `{ fast: true }` when quicker generation matters more than file
size, such as during interactive previews:

```ts
const preview = await buildSpriteSheet(icons, 2, { fast: true });
```

## SVG limitations

- External resources such as linked images and web fonts are not loaded.
- SVG `<text>` is not supported because fonts are not bundled. Text elements
  may parse successfully but are not rendered.

## Development

Install the repository's pinned Rust toolchain, the wasm target, the matching
wasm-bindgen CLI, Binaryen's `wasm-opt`, Node, and pnpm:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.125 --locked
pnpm install
pnpm build
pnpm typecheck
pnpm test
pnpm bench
pnpm --filter @kartore/spritore pack --dry-run
```

The build stops if `wasm-bindgen` has a different version or `wasm-opt` is not
available. Generated files under `js/pkg/` and `js/dist/` are built before
testing and publishing.

## License

Licensed under either the Apache License, Version 2.0 or the MIT License, at
your option.
