import initWasm, {
	buildSpriteSheet as buildSpriteSheetWasm,
	renderIcon as renderIconWasm,
	type InitInput,
} from "../pkg/spritore_wasm.js";
import type {
	IconSource,
	ImageMetadata,
	RenderedIcon,
	SpriteIndexEntry,
	SpriteSheet,
	WasmInput,
} from "./types.js";

interface WasmRenderedIcon extends ImageMetadata {
	id: string;
	width: number;
	height: number;
	pixels: Uint8Array;
	sdf: boolean;
}

/** wasm に渡す描画オプション (sdf とメタデータ)。undefined のキーは渡さない */
export interface RenderOptionsInput extends ImageMetadata {
	readonly sdf?: boolean;
}

const METADATA_KEYS = [
	"stretchX",
	"stretchY",
	"content",
	"textFitWidth",
	"textFitHeight",
] as const satisfies readonly (keyof ImageMetadata)[];

function definedEntries<T extends object>(value: T): Partial<T> {
	return Object.fromEntries(
		Object.entries(value).filter(([, entry]) => entry !== undefined),
	) as Partial<T>;
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
	options: RenderOptionsInput,
	wasmInput: () => WasmInput,
): Promise<RenderedIcon> {
	await initializeWasm(wasmInput);
	const wasmOptions = definedEntries({
		sdf: options.sdf,
		stretchX: options.stretchX,
		stretchY: options.stretchY,
		content: options.content,
		textFitWidth: options.textFitWidth,
		textFitHeight: options.textFitHeight,
	});
	const rendered = renderIconWasm(
		id,
		svg,
		pixelRatio,
		Object.keys(wasmOptions).length === 0 ? undefined : wasmOptions,
	) as WasmRenderedIcon;
	const metadata = Object.fromEntries(
		METADATA_KEYS.filter((key) => rendered[key] !== undefined).map((key) => [
			key,
			rendered[key],
		]),
	) as ImageMetadata;
	return Object.freeze({
		id: rendered.id,
		width: rendered.width,
		height: rendered.height,
		pixels: rendered.pixels,
		sdf: rendered.sdf,
		...metadata,
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
