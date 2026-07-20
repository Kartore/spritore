# spritore-core

`spritore-core` rasterizes SVG icons and builds MapLibre-compatible PNG sprite
sheets with JSON indexes. Use it when a Rust application already owns the SVG
input and wants to handle generated assets in memory.

It supports two common workflows:

- Render one SVG to straight-alpha RGBA pixels for MapLibre's `map.addImage`.
- Combine multiple rendered icons into a PNG sprite sheet and index.

Most Rust applications can use the `spritore` crate, which re-exports this API.
Depend on `spritore-core` directly when you do not need the `spritore` command.

## Install

```sh
cargo add spritore-core
```

Or add it to `Cargo.toml`:

```toml
[dependencies]
spritore-core = "0.2.0"
```

## Build a sprite sheet

```rust
use spritore_core::{
	BuildOptions, build_sprite_sheet, index_to_json, render_icon,
};

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

The API operates on data supplied by the caller. Read SVG files before calling
`render_icon`, and write `SpriteSheet::png` and `index_to_json(...)` wherever
your application stores sprite assets.

Use `BuildOptions { fast: true }` for quicker preview generation. The default
mode prioritizes a smaller PNG for final assets.

## API overview

- `render_icon` turns one SVG string into a `RenderedIcon`.
- `build_sprite_sheet` combines rendered icons into a `SpriteSheet`.
- `index_to_json` converts the returned index into ready-to-write MapLibre JSON.

Duplicate icon IDs, empty icon collections, invalid SVG, and zero-sized icons
are reported through `Error`.

## SVG limitations

- External resources such as linked images and web fonts are not loaded.
- SVG `<text>` is not supported because fonts are not bundled. Text elements
  may parse successfully but are not rendered.

## License

Licensed under either Apache-2.0 or MIT, at your option.
