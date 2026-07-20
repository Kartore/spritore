import initWasm, {
	buildSpriteSheet as buildSpriteSheetWasm,
	renderIcon as renderIconWasm,
} from "./pkg/spritore_wasm.js";

let initialization;
let initialized = false;

/**
 * Loads the package before rendering icons or sprite sheets.
 *
 * Omit `input` during normal browser use. Repeated calls return the first
 * initialization promise.
 *
 * @param {BufferSource | Promise<Response> | Response} [input]
 * @returns {Promise<void>}
 */
export function init(input) {
	if (initialization === undefined) {
		const wasmInput =
			input === undefined
				? new URL("./pkg/spritore_wasm_bg.wasm", import.meta.url)
				: input;
		initialization = Promise.resolve(wasmInput)
			.then((resolvedInput) => initWasm({ module_or_path: resolvedInput }))
			.then(() => {
				initialized = true;
			});
	}
	return initialization;
}

/**
 * Rasterizes one SVG into straight-alpha RGBA pixels for MapLibre.
 *
 * @param {string} id Identifier to use as the sprite index key.
 * @param {string} svg SVG source text.
 * @param {number} pixelRatio Output pixel ratio, typically 1 or 2.
 * @returns {{id: string, width: number, height: number, pixels: Uint8Array}}
 * @throws {Error} If initialization has not completed or the SVG is invalid.
 */
export function renderIcon(id, svg, pixelRatio) {
	assertInitialized();
	return renderIconWasm(id, svg, pixelRatio);
}

/**
 * Builds a PNG sprite sheet and MapLibre index from SVG icon sources.
 *
 * @param {{id: string, svg: string}[]} icons SVG sources and their index keys.
 * @param {number} pixelRatio Output pixel ratio, typically 1 or 2.
 * @param {{fast?: boolean}} [options] Set `fast` to prioritize generation speed.
 * @returns {{png: Uint8Array, index: Record<string, {x: number, y: number, width: number, height: number, pixelRatio: number}>, indexJson: string}}
 * @throws {Error} If initialization has not completed or an input is invalid.
 */
export function buildSpriteSheet(icons, pixelRatio, options) {
	assertInitialized();
	return buildSpriteSheetWasm(icons, pixelRatio, options);
}

function assertInitialized() {
	if (!initialized) {
		throw new Error("@kartore/spritore is not initialized; call and await init() first");
	}
}
