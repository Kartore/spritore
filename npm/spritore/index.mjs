import initWasm, {
	buildSpriteSheet as buildSpriteSheetWasm,
	renderIcon as renderIconWasm,
} from "./pkg/spritore_wasm.js";

let initialization;
let initialized = false;

/**
 * Initializes the bundled WebAssembly module exactly once.
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

/** Rasterizes one SVG icon into straight-alpha RGBA pixels. */
export function renderIcon(id, svg, pixelRatio) {
	assertInitialized();
	return renderIconWasm(id, svg, pixelRatio);
}

/** Builds a deterministic PNG sprite sheet from SVG icon sources. */
export function buildSpriteSheet(icons, pixelRatio, options) {
	assertInitialized();
	return buildSpriteSheetWasm(icons, pixelRatio, options);
}

function assertInitialized() {
	if (!initialized) {
		throw new Error("@kartore/spritore is not initialized; call and await init() first");
	}
}
