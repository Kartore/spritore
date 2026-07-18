# @kartore/spritore

Deterministic WebAssembly sprite generation for MapLibre GL styles. The same
SVG inputs, pixel ratio, and compression mode produce the same PNG and index
bytes as the Rust `spritore-core` pipeline.

## Browser

```js
import { buildSpriteSheet, init, renderIcon } from "@kartore/spritore";

await init();
const icon = renderIcon("marker", markerSvg, 2);
const sheet = buildSpriteSheet([{ id: "marker", svg: markerSvg }], 2);
```

## Node

```js
import { buildSpriteSheet, init } from "@kartore/spritore/node";

await init();
const sheet = buildSpriteSheet(icons, 1, { fast: true });
```

`fast: true` uses miniz for previews. The default uses slower Zopfli
compression for smaller downloads.

## Development

Install the repository's pinned Rust toolchain, the wasm target, the matching
wasm-bindgen CLI, Binaryen's `wasm-opt`, Node, and pnpm:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.125 --locked
pnpm -C npm/spritore build
pnpm -C npm/spritore test
pnpm -C npm/spritore bench
```

The build stops if `wasm-bindgen` has a different version or `wasm-opt` is not
available. The generated `pkg/` directory is intentionally not committed.

SVG external resources and fonts are not loaded. SVG `<text>` is therefore not
supported in this release.
