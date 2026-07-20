/** A rasterized SVG icon that can be added directly to a MapLibre map. */
export type RenderedIcon = {
	/** The caller-provided identifier. */
	id: string;
	/** The rendered width in pixels. */
	width: number;
	/** The rendered height in pixels. */
	height: number;
	/** Straight-alpha RGBA data for MapLibre's `map.addImage` data field. */
	pixels: Uint8Array;
};

/** The location and dimensions of one icon in a MapLibre sprite sheet. */
export type SpriteIndexEntry = {
	/** The icon's left coordinate in the PNG. */
	x: number;
	/** The icon's top coordinate in the PNG. */
	y: number;
	/** The icon's width in pixels. */
	width: number;
	/** The icon's height in pixels. */
	height: number;
	/** The pixel ratio represented by the sprite sheet. */
	pixelRatio: number;
};

/** Generated PNG data and its MapLibre sprite index. */
export type SpriteSheet = {
	/** The complete PNG file. */
	png: Uint8Array;
	/** Sprite entries keyed by the input icon IDs. */
	index: Record<string, SpriteIndexEntry>;
	/** Ready-to-write MapLibre sprite index JSON, including a trailing newline. */
	indexJson: string;
};

/** Options for building a sprite sheet. */
export type BuildOptions = {
	/** Prioritizes generation speed over PNG file size. Defaults to `false`. */
	fast?: boolean;
};

/**
 * Loads the package before rendering icons or sprite sheets.
 *
 * Omit `input` during normal browser or Node use. Repeated calls return the
 * first initialization promise.
 *
 * @param input Optional WebAssembly bytes or response supplied by the caller.
 */
export function init(
	input?: BufferSource | Promise<Response> | Response,
): Promise<void>;

/**
 * Rasterizes one SVG into straight-alpha RGBA pixels.
 *
 * The returned dimensions and pixel data are suitable for MapLibre's
 * `map.addImage`. Call and await `init()` first.
 *
 * @param id Identifier to use as the sprite index key.
 * @param svg SVG source text.
 * @param pixelRatio Output pixel ratio, typically 1 or 2.
 */
export function renderIcon(
	id: string,
	svg: string,
	pixelRatio: number,
): RenderedIcon;

/**
 * Rasterizes SVG icon sources and builds a PNG sprite sheet with its MapLibre
 * index.
 *
 * Each item needs an `id` for the index key and an `svg` string. Call and await
 * `init()` first. Duplicate IDs, an empty array, and invalid SVG are errors.
 *
 * @param icons SVG sources and their index keys.
 * @param pixelRatio Output pixel ratio, typically 1 or 2.
 * @param options Set `fast` when generation speed matters more than PNG size.
 */
export function buildSpriteSheet(
	icons: { id: string; svg: string }[],
	pixelRatio: number,
	options?: BuildOptions,
): SpriteSheet;
