import { Buffer } from "node:buffer";

import {
	buildSpriteSheet,
	renderIcon,
	type RenderedIcon,
} from "@kartore/spritore/node";

async function nodeExample(svg: string, wasm: Buffer): Promise<void> {
	const icon: RenderedIcon = await renderIcon("marker", svg, 1, { wasm });
	const sheet = await buildSpriteSheet([{ id: icon.id, svg }], 1, {
		fast: true,
	});
	Buffer.from(sheet.png);
}

void nodeExample;
