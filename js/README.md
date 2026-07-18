# @kartore/spritore

WebAssembly sprite generation for [MapLibre GL](https://maplibre.org/) styles,
with browser and Node entry points.

The package rasterizes individual SVG icons and builds complete PNG sprite
sheets with their JSON indexes.

## Install

```sh
pnpm add @kartore/spritore
```

Or with npm:

```sh
npm install @kartore/spritore
```

## Browser

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

With no argument, `init()` resolves the wasm file bundled with the package.
Initialization is cached, so repeated calls return the same promise. Calling
`renderIcon` or `buildSpriteSheet` before initialization throws an `Error`.

## Node

Use the Node entry point so the bundled wasm is read from the filesystem and
passed to the same initializer used by browsers:

```js
import { readFile, writeFile } from "node:fs/promises";

import { buildSpriteSheet, init } from "@kartore/spritore/node";

await init();

const markerSvg = await readFile("marker.svg", "utf8");
const icons = [{ id: "marker", svg: markerSvg }];
const sprite = buildSpriteSheet(icons, 1);
await writeFile("sprite.png", sprite.png);
await writeFile("sprite.json", sprite.indexJson);
```

Write `indexJson` directly when producing a sprite index. It is the
preformatted JSON emitted by the Rust core, including its trailing newline.

## API

### `init(input?)`

Initializes the WebAssembly module exactly once. The optional input can be a
`BufferSource`, `Response`, or `Promise<Response>`; normally it should be
omitted.

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

The dimensions and pixel buffer can be passed directly to MapLibre's
`map.addImage` API as shown above.

### `buildSpriteSheet(icons, pixelRatio, options?)`

Builds a complete MapLibre sprite sheet:

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

Input order does not affect the output. Duplicate IDs and empty icon arrays are
errors. Pixel-identical icons are packed once while keeping an index entry for
every ID.

## Compression modes

The default mode uses Zopfli for smaller downloads. It is intended for final
assets and may be substantially slower than preview generation.

Pass `{ fast: true }` to use miniz instead:

```js
const preview = buildSpriteSheet(icons, 2, { fast: true });
```

The two modes produce different PNG encodings.

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
available. Generated files under `js/pkg/` are intentionally not committed;
they are built before testing and publishing.

## License

Licensed under either the Apache License, Version 2.0 or the MIT License, at
your option.
