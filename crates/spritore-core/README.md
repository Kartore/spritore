# spritore-core

Low-level Rust engine for generating MapLibre-compatible PNG sprite sheets and
JSON indexes from SVG icons.

Most applications should depend on the `spritore` facade crate. Use
`spritore-core` directly when building another integration layer or when the
smallest dependency surface is preferred.

## Install

```sh
cargo add spritore-core
```

Or add it to `Cargo.toml`:

```toml
[dependencies]
spritore-core = "0.1.0"
```

## Usage

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

The crate performs no filesystem access itself. Callers provide SVG strings
and decide where generated assets are written.

## License

Licensed under either Apache-2.0 or MIT, at your option.
