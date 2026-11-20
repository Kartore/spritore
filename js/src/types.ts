/** An SVG icon source and the identifier used in a sprite index. */
export interface IconSource {
	/** The identifier used as the sprite index key. */
	readonly id: string;
	/** SVG source text. */
	readonly svg: string;
	/**
	 * Renders the icon as a MapLibre SDF icon and marks its index entry with
	 * `sdf: true`. Defaults to `false`.
	 */
	readonly sdf?: boolean;
}

/** A rasterized SVG icon that can be added directly to a MapLibre map. */
export interface RenderedIcon {
	/** The caller-provided identifier. */
	readonly id: string;
	/** The rendered width in pixels. */
	readonly width: number;
	/** The rendered height in pixels. */
	readonly height: number;
	/**
	 * Straight-alpha RGBA data for MapLibre's `map.addImage` data field. For
	 * SDF icons, RGB is zero and alpha holds the distance field.
	 */
	readonly pixels: Uint8Array;
	/**
	 * `true` when `pixels` hold a signed distance field. Pass it to
	 * `map.addImage(id, image, { sdf })`.
	 */
	readonly sdf: boolean;
}

/** The location and dimensions of one icon in a MapLibre sprite sheet. */
export interface SpriteIndexEntry {
	/** The icon's left coordinate in the PNG. */
	readonly x: number;
	/** The icon's top coordinate in the PNG. */
	readonly y: number;
	/** The icon's width in pixels. */
	readonly width: number;
	/** The icon's height in pixels. */
	readonly height: number;
	/** The pixel ratio represented by the sprite sheet. */
	readonly pixelRatio: number;
	/** Present and `true` only for MapLibre SDF icons. */
	readonly sdf?: true;
}

/** Generated PNG data and its MapLibre sprite index. */
export interface SpriteSheet {
	/** The complete PNG file. */
	readonly png: Uint8Array;
	/** Frozen sprite entries keyed by the input icon IDs. */
	readonly index: Readonly<Record<string, SpriteIndexEntry>>;
	/** Ready-to-write MapLibre sprite index JSON, including a trailing newline. */
	readonly indexJson: string;
}

/** A custom source used to initialize the bundled WebAssembly module. */
export type WasmSource = string | URL | ArrayBuffer | Uint8Array;

/** A custom source, or a promise for one, used to initialize WebAssembly. */
export type WasmInput = WasmSource | Promise<WasmSource>;

/** Options for rendering one icon. */
export interface RenderIconOptions {
	/**
	 * Renders a signed distance field for a MapLibre SDF icon. Only the
	 * icon's alpha is kept, and the image gains 3 logical pixels of padding on
	 * every side (scaled by the pixel ratio). Defaults to `false`.
	 */
	readonly sdf?: boolean;
	/**
	 * Overrides the bundled WebAssembly input. Only the first API call
	 * initializes the module; later calls reuse that initialization.
	 */
	readonly wasm?: WasmInput;
}

/** Options for building a sprite sheet. */
export interface BuildOptions {
	/** Prioritizes generation speed over PNG file size. Defaults to `false`. */
	readonly fast?: boolean;
	/**
	 * Overrides the bundled WebAssembly input. Only the first API call
	 * initializes the module; later calls reuse that initialization.
	 */
	readonly wasm?: WasmInput;
}
