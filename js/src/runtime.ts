import initWasm, {
	buildSpriteSheet as buildSpriteSheetWasm,
	renderIcon as renderIconWasm,
	type InitInput,
} from "../pkg/spritore_wasm.js";
import type {
	IconSource,
	RenderedIcon,
	SpriteIndexEntry,
	SpriteSheet,
	WasmInput,
} from "./types.js";

interface WasmRenderedIcon {
	id: string;
	width: number;
	height: number;
	pixels: Uint8Array;
}

interface WasmSpriteSheet {
	png: Uint8Array;
	index: Record<string, SpriteIndexEntry>;
	indexJson: string;
}

let initialization: Promise<void> | undefined;

export async function renderIconWithWasm(
	id: string,
	svg: string,
	pixelRatio: number,
	wasmInput: () => WasmInput,
): Promise<RenderedIcon> {
	await initializeWasm(wasmInput);
	const rendered = renderIconWasm(id, svg, pixelRatio) as WasmRenderedIcon;
	return Object.freeze({
		id: rendered.id,
		width: rendered.width,
		height: rendered.height,
		pixels: rendered.pixels,
	});
}

export async function buildSpriteSheetWithWasm(
	icons: readonly IconSource[],
	pixelRatio: number,
	fast: boolean | undefined,
	wasmInput: () => WasmInput,
): Promise<SpriteSheet> {
	await initializeWasm(wasmInput);
	const wasmOptions = fast === undefined ? undefined : { fast };
	const sheet = buildSpriteSheetWasm(
		icons,
		pixelRatio,
		wasmOptions,
	) as WasmSpriteSheet;
	return Object.freeze({
		png: sheet.png,
		index: freezeSpriteIndex(sheet.index),
		indexJson: sheet.indexJson,
	});
}

function initializeWasm(wasmInput: () => WasmInput): Promise<void> {
	if (initialization === undefined) {
		let attempt: Promise<void>;
		attempt = Promise.resolve()
			.then(wasmInput)
			.then((input) =>
				initWasm({ module_or_path: input as InitInput }),
			)
			.then(() => undefined)
			.catch((error: unknown) => {
				if (initialization === attempt) {
					initialization = undefined;
				}
				throw error;
			});
		initialization = attempt;
	}
	return initialization;
}

function freezeSpriteIndex(
	index: Record<string, SpriteIndexEntry>,
): Readonly<Record<string, SpriteIndexEntry>> {
	for (const entry of Object.values(index)) {
		Object.freeze(entry);
	}
	return Object.freeze(index);
}
