# spritore

Deterministic sprite generator for [MapLibre GL](https://maplibre.org/) styles.
Rust core with JS/WASM bindings (browser & Node) and a CLI.

Unlike existing sprite tools, spritore is built around three guarantees:

1. **Byte-deterministic output** — the same input SVGs always produce the same
   `sprite.png` / `sprite.json` bytes, on every platform. Sprite assets become
   diffable and reviewable in git.
2. **JS/WASM bindings** — the exact same generator runs in the browser and in
   Node, not just as a CLI. Built for live style editors
   ([Kartore](https://github.com/Kartore)).
3. **Size-optimized output** — lossless palette reduction, PNG filter
   selection, zopfli compression, and duplicate-icon deduplication by default.

> **Status: early development.** The design plan lives in
> [docs/plan.md](docs/plan.md). This Rust rewrite is published as
> [`@kartore/spritore`](https://www.npmjs.com/package/@kartore/spritore) —
> versions `0.0.x` are the previous TypeScript implementation, which lives in
> the git history of this repository.

## Planned interfaces

```
spritore build <svg-dir> -o <out-dir> [--name sprite] [--ratio 1,2] [--fast]
```

```ts
import { init, buildSpriteSheet } from '@kartore/spritore';

await init();
const { png, index } = buildSpriteSheet(
	[{ id: 'airport', svg: '<svg …>' }],
	2
);
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
