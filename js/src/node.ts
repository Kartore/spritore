import { readFileSync } from "node:fs";

import {
	buildSpriteSheetWithWasm,
	renderIconWithWasm,
} from "./runtime.js";
import type {
	BuildOptions,
	IconSource,
	RenderedIcon,
	RenderIconOptions,
	SpriteSheet,
} from "./types.js";

export type {
	BuildOptions,
	IconSource,
	RenderedIcon,
	RenderIconOptions,
	SpriteIndexEntry,
	SpriteSheet,
	WasmInput,
	WasmSource,
} from "./types.js";

const bundledWasm = new URL(
	"../pkg/spritore_wasm_bg.wasm",
	import.meta.url,
);

/**
 * Rasterizes one SVG into straight-alpha RGBA pixels for MapLibre.
 *
 * The bundled WebAssembly file is loaded automatically on the first API call.
 * The returned plain object is frozen and owns its pixel data.
 *
 * @throws An `Error` when the SVG is invalid or WebAssembly initialization
 * fails.
 */
export function renderIcon(
	id: string,
	svg: string,
	pixelRatio: number,
	options: RenderIconOptions = {},
): Promise<RenderedIcon> {
	const wasm = options.wasm;
	return renderIconWithWasm(
		id,
		svg,
		pixelRatio,
		() => wasm ?? readFileSync(bundledWasm),
	);
}

/**
 * Rasterizes SVG icon sources and builds a PNG sprite sheet with its MapLibre
 * index.
 *
 * The bundled WebAssembly file is loaded automatically on the first API call.
 * Duplicate IDs, an empty array, and invalid SVG are errors. The returned
 * plain object, index, and index entries are frozen.
 */
export function buildSpriteSheet(
	icons: readonly IconSource[],
	pixelRatio: number,
	options: BuildOptions = {},
): Promise<SpriteSheet> {
	const { fast, wasm } = options;
	return buildSpriteSheetWithWasm(
		icons,
		pixelRatio,
		fast,
		() => wasm ?? readFileSync(bundledWasm),
	);
}
