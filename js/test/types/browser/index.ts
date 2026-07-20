import * as spritore from "@kartore/spritore";
import {
	buildSpriteSheet,
	renderIcon,
	type BuildOptions,
	type RenderedIcon,
	type SpriteSheet,
} from "@kartore/spritore";

declare const svg: string;
declare const options: BuildOptions;

async function browserExample(): Promise<void> {
	const icon: RenderedIcon = await renderIcon("marker", svg, 2);
	const sheet: SpriteSheet = await buildSpriteSheet(
		[{ id: icon.id, svg }],
		2,
		options,
	);

	icon.pixels[0] = 0;
	void sheet.png;
	void sheet.index[icon.id]?.width;

	// @ts-expect-error Result properties are read-only.
	icon.width = 1;
	// @ts-expect-error Sprite index entries are read-only.
	sheet.index[icon.id] = sheet.index[icon.id]!;
	// @ts-expect-error Rendering is a standalone function.
	icon.render();
	// @ts-expect-error Initialization is not part of the public API.
	spritore.init();
}

void browserExample;
