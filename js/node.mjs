import { readFileSync } from "node:fs";

import {
	buildSpriteSheet,
	init as initBrowser,
	renderIcon,
} from "./index.mjs";

let initialization;

/**
 * Loads the package before rendering icons or sprite sheets in Node.
 *
 * With no input, Node loads the WebAssembly file included in the package.
 * Repeated calls return the first initialization promise.
 *
 * @param {BufferSource | Promise<Response> | Response} [input]
 * @returns {Promise<void>}
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
