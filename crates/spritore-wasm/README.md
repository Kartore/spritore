# spritore-wasm

`spritore-wasm` exposes WebAssembly functions for rasterizing SVG icons and
building MapLibre-compatible PNG sprite sheets with JSON indexes.

JavaScript and TypeScript applications should install
[`@kartore/spritore`](../../js/README.md). That package provides browser and
Node entry points, initialization helpers, type declarations, and the
`spritore` command-line executable. This crate is built for that package and is
not distributed as a standalone crates.io library.

## Exports

- `renderIcon(id, svg, pixelRatio)` returns one icon as straight-alpha RGBA
  pixels suitable for MapLibre's `map.addImage`.
- `buildSpriteSheet(icons, pixelRatio, options?)` returns PNG bytes, a parsed
  MapLibre index, and ready-to-write index JSON.

Use `{ fast: true }` when preview generation speed matters more than PNG file
size. The default mode is intended for final assets.

For installation, CLI usage, browser and Node examples, API details, and SVG
limitations, see the [npm package README](../../js/README.md).

## Development

Build and test the package from the repository root:

```sh
pnpm -C js build
pnpm -C js test
```

The required tool versions and setup commands are listed in the
[npm package development guide](../../js/README.md#development).

## License

Licensed under either Apache-2.0 or MIT, at your option.
