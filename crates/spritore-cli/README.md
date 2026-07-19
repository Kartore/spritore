# spritore

MapLibre sprite generation for Rust, with a native command-line interface.

## CLI

```sh
cargo install spritore
spritore build ./icons -o ./public/sprites
```

The complete command is:

```text
spritore build <svg-dir> -o <out-dir> [--name sprite] [--ratio 1,2] [--fast] [--skip-invalid]
```

- `--name` changes the output basename.
- `--ratio` selects comma-separated pixel ratios from 1 to 255.
- `--fast` uses faster miniz compression instead of Zopfli.
- `--skip-invalid` reports and excludes SVG parse errors.

## Rust API

The library target re-exports the sprite API from `spritore-core`. To use the
API without compiling the CLI dependency, disable default features:

```sh
cargo add spritore --no-default-features
```

Or add it to `Cargo.toml`:

```toml
[dependencies]
spritore = { version = "0.1.0", default-features = false }
```

```rust
use spritore::{BuildOptions, build_sprite_sheet, render_icon};

fn main() -> Result<(), spritore::Error> {
	let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
		<circle cx="8" cy="8" r="6" fill="#4264fb" />
	</svg>"##;
	let icons = [render_icon("marker", svg, 1)?];
	let sheet = build_sprite_sheet(&icons, 1, BuildOptions::default())?;
	println!("{} icons", sheet.index.len());
	Ok(())
}
```

## Features

- `cli` (default): builds the `spritore` executable and enables argument
  parsing with `clap`.
- With default features disabled, only the Rust library API and
  `spritore-core` dependency remain.

## License

Licensed under either Apache-2.0 or MIT, at your option.
