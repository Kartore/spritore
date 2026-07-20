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

Call and await `init()` before using the rendering functions:

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
map.addImage("marker", {
	width: marker.width,
	height: marker.height,
	data: marker.pixels,
});

const sprite = buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	2,
);
```

With no argument, `init()` loads the WebAssembly file included in the package.
Initialization is cached, so repeated calls return the first promise. Calling
`renderIcon` or `buildSpriteSheet` before initialization throws an `Error`.

## Node

Use the `/node` entry point so `init()` can load the packaged WebAssembly file
from the filesystem:

```js
import { readFile, writeFile } from "node:fs/promises";

import { buildSpriteSheet, init } from "@kartore/spritore/node";

await init();

const markerSvg = await readFile("marker.svg", "utf8");
const sprite = buildSpriteSheet(
	[{ id: "marker", svg: markerSvg }],
	1,
);

await writeFile("sprite.png", sprite.png);
await writeFile("sprite.json", sprite.indexJson);
```

Write `indexJson` directly when creating a sprite index. The `index` property
contains the parsed object for applications that need to inspect or transform
entries.

## API

### `init(input?)`

Loads the package before rendering. The optional input can be a `BufferSource`,
`Response`, or `Promise<Response>`; normal browser and Node usage can omit it.
Repeated calls reuse the first initialization promise.

### `renderIcon(id, svg, pixelRatio)`

Rasterizes one SVG and returns straight-alpha RGBA pixels:

```ts
type RenderedIcon = {
	id: string;
	width: number;
	height: number;
	pixels: Uint8Array;
};
```

Use the returned `width`, `height`, and `pixels` as MapLibre's
`map.addImage` image data. A pixel ratio of `1` or `2` covers typical
standard- and high-density previews.

### `buildSpriteSheet(icons, pixelRatio, options?)`

Rasterizes a collection of SVG icons and returns complete MapLibre sprite
assets:

```ts
type SpriteSheet = {
	png: Uint8Array;
	index: Record<string, {
		x: number;
		y: number;
		width: number;
		height: number;
		pixelRatio: number;
	}>;
	indexJson: string;
};

type BuildOptions = {
	fast?: boolean;
};
```

Each input item has an `id` used as its index key and an `svg` string. Duplicate
IDs, empty icon arrays, invalid SVG, and zero-sized icons are errors.

## Build modes

The default mode prioritizes smaller PNG files and is intended for final
assets. Pass `{ fast: true }` when quicker generation matters more than file
size, such as during interactive previews:

```js
const preview = buildSpriteSheet(icons, 2, { fast: true });
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
pnpm -C js install
pnpm -C js build
pnpm -C js test
pnpm -C js bench
pnpm -C js pack --dry-run
```

The build stops if `wasm-bindgen` has a different version or `wasm-opt` is not
available. Generated files under `js/pkg/` are built before testing and
publishing.

## License

Licensed under either the Apache License, Version 2.0 or the MIT License, at
your option.
