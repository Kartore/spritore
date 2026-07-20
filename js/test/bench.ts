import { readFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";

import {
	buildSpriteSheet,
	renderIcon,
	type IconSource,
} from "@kartore/spritore/node";

// Keep the benchmark inert if it is passed to Node's test runner directly.
// Run it with `node test/bench.ts` or `pnpm bench`.
if (process.env.NODE_TEST_CONTEXT === undefined) {
	await main();
}

async function main(): Promise<void> {
	const fixtureDirectory = new URL(
		"../../crates/spritore-core/tests/fixtures/",
		import.meta.url,
	);
	const fixtureNames = ["dot", "pin", "grad", "line", "dot-copy", "spectrum"];
	const fixtureSources: IconSource[] = await Promise.all(
		fixtureNames.map(async (id) => ({
			id,
			svg: await readFile(new URL(`${id}.svg`, fixtureDirectory), "utf8"),
		})),
	);
	const icons: IconSource[] = Array.from({ length: 200 }, (_, index) => {
		const fixture = fixtureSources[index % fixtureSources.length]!;
		return {
			id: `${fixture.id}-${String(index).padStart(3, "0")}`,
			svg: fixture.svg,
		};
	});

	const warmup = fixtureSources[0]!;
	await renderIcon("warmup", warmup.svg, 2);

	let started = performance.now();
	for (const icon of icons) {
		await renderIcon(icon.id, icon.svg, 2);
	}
	const renderMilliseconds = performance.now() - started;

	started = performance.now();
	const fast = await buildSpriteSheet(icons, 2, { fast: true });
	const fastMilliseconds = performance.now() - started;

	started = performance.now();
	const zopfli = await buildSpriteSheet(icons, 2);
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

function round(value: number): number {
	return Math.round(value * 10) / 10;
}
