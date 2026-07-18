import { readFileSync } from "node:fs";

import {
	buildSpriteSheet,
	init as initBrowser,
	renderIcon,
} from "./index.mjs";

let initialization;

/**
 * Initializes the bundled WebAssembly module exactly once.
 *
 * With no input, Node reads the package's single bundled wasm file and passes
 * its bytes to the same initializer used by the browser entry point.
 */
export function init(input) {
	if (initialization === undefined) {
		const wasmInput =
			input === undefined
				? readFileSync(new URL("./pkg/spritore_wasm_bg.wasm", import.meta.url))
				: input;
		initialization = initBrowser(wasmInput);
	}
	return initialization;
}

export { buildSpriteSheet, renderIcon };
