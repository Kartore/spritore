/** A rendered icon with straight-alpha RGBA pixels. */
export type RenderedIcon = {
	id: string;
	width: number;
	height: number;
	/** Suitable for MapLibre's `map.addImage` data field. */
	pixels: Uint8Array;
};

/** One MapLibre sprite index entry. */
export type SpriteIndexEntry = {
	x: number;
	y: number;
	width: number;
	height: number;
	pixelRatio: number;
};

/** A PNG sprite sheet and its MapLibre index. */
export type SpriteSheet = {
	png: Uint8Array;
	index: Record<string, SpriteIndexEntry>;
	/** Deterministic core JSON, including its trailing newline. */
	indexJson: string;
};

/** Options controlling sprite sheet construction. */
export type BuildOptions = {
	/** Uses miniz instead of Zopfli when true. Defaults to false. */
	fast?: boolean;
};

/** Initializes the bundled WebAssembly module exactly once. */
export function init(
	input?: BufferSource | Promise<Response> | Response,
): Promise<void>;

/** Rasterizes one SVG icon into straight-alpha RGBA pixels. */
export function renderIcon(
	id: string,
	svg: string,
	pixelRatio: number,
): RenderedIcon;

/** Builds a deterministic PNG sprite sheet from SVG icon sources. */
export function buildSpriteSheet(
	icons: { id: string; svg: string }[],
	pixelRatio: number,
	options?: BuildOptions,
): SpriteSheet;
