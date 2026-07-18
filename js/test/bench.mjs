import { readFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";

import {
	buildSpriteSheet,
	init,
	renderIcon,
} from "@kartore/spritore/node";

// Keep the benchmark inert if it is passed to Node's test runner directly.
// Run it with `node test/bench.mjs` or `pnpm bench`.
if (process.env.NODE_TEST_CONTEXT === undefined) {
	await main();
}

async function main() {
	const fixtureDirectory = new URL(
		"../../crates/spritore-core/tests/fixtures/",
		import.meta.url,
	);
	const fixtureNames = ["dot", "pin", "grad", "line", "dot-copy", "spectrum"];
	const fixtureSources = await Promise.all(
		fixtureNames.map(async (id) => ({
			id,
			svg: await readFile(new URL(`${id}.svg`, fixtureDirectory), "utf8"),
		})),
	);
	const icons = Array.from({ length: 200 }, (_, index) => {
		const fixture = fixtureSources[index % fixtureSources.length];
		return {
			id: `${fixture.id}-${String(index).padStart(3, "0")}`,
			svg: fixture.svg,
		};
	});

	await init();

	let started = performance.now();
	for (const icon of icons) {
		renderIcon(icon.id, icon.svg, 2);
	}
	const renderMilliseconds = performance.now() - started;

	started = performance.now();
	const fast = buildSpriteSheet(icons, 2, { fast: true });
	const fastMilliseconds = performance.now() - started;

	started = performance.now();
	const zopfli = buildSpriteSheet(icons, 2);
	const zopfliMilliseconds = performance.now() - started;

	console.log(
		JSON.stringify(
			{
				icons: icons.length,
				pixelRatio: 2,
				renderIconMilliseconds: round(renderMilliseconds),
				buildFastMilliseconds: round(fastMilliseconds),
				buildZopfliMilliseconds: round(zopfliMilliseconds),
				fastPngBytes: fast.png.length,
				zopfliPngBytes: zopfli.png.length,
			},
			null,
			2,
		),
	);
}

function round(value) {
	return Math.round(value * 10) / 10;
}
