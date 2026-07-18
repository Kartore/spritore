import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import {
	buildSpriteSheet,
	init,
	renderIcon,
} from "@kartore/spritore/node";

const fixtureDirectory = new URL(
	"../../../crates/spritore-core/tests/fixtures/",
	import.meta.url,
);
const goldenDirectory = new URL(
	"../../../crates/spritore-core/tests/golden/",
	import.meta.url,
);
const fixtureNames = ["dot", "pin", "grad", "line", "dot-copy", "spectrum"];
const fixtures = await Promise.all(
	fixtureNames.map(async (id) => ({
		id,
		svg: await readFile(new URL(`${id}.svg`, fixtureDirectory), "utf8"),
	})),
);

test("Node entry API", async (context) => {
	await context.test("render and build fail clearly before init", () => {
		assert.throws(() => renderIcon("dot", "<svg/>", 1), /not initialized/);
		assert.throws(() => buildSpriteSheet([], 1), /not initialized/);
	});

	await context.test("init caches one promise", async () => {
		const firstInitialization = init();
		const secondInitialization = init();
		assert.strictEqual(secondInitialization, firstInitialization);
		await firstInitialization;
	});

	await context.test("buildSpriteSheet matches every core golden byte", async () => {
		for (const pixelRatio of [1, 2]) {
			const suffix = pixelRatio === 1 ? "" : "@2x";
			const expectedIndex = await readFile(
				new URL(`sprite${suffix}.json`, goldenDirectory),
			);

			for (const fast of [true, false]) {
				const kind = fast ? ".fast" : "";
				const expectedPng = await readFile(
					new URL(`sprite${suffix}${kind}.png`, goldenDirectory),
				);
				const sheet = buildSpriteSheet(
					fixtures,
					pixelRatio,
					fast ? { fast: true } : undefined,
				);

				assert.equal(Buffer.compare(Buffer.from(sheet.png), expectedPng), 0);
				assert.equal(
					Buffer.compare(Buffer.from(sheet.indexJson, "utf8"), expectedIndex),
					0,
				);
				assert.ok(sheet.png instanceof Uint8Array);
				assert.ok(!(sheet.index instanceof Map));
				assert.deepEqual(sheet.index, JSON.parse(sheet.indexJson));
			}
		}
	});

	await context.test("renderIcon returns straight RGBA bytes in a Uint8Array", () => {
		const dot = fixtures.find((fixture) => fixture.id === "dot");
		const rendered = renderIcon(dot.id, dot.svg, 1);

		assert.equal(rendered.id, "dot");
		assert.equal(rendered.width, 15);
		assert.equal(rendered.height, 15);
		assert.equal(rendered.pixels.length, 15 * 15 * 4);
		assert.ok(rendered.pixels instanceof Uint8Array);
	});

	await context.test("invalid SVG errors include the icon id", () => {
		assert.throws(
			() => renderIcon("broken-icon", "not svg", 1),
			(error) => error instanceof Error && error.message.includes("broken-icon"),
		);
	});

	await context.test("zero ratio, duplicate, and empty inputs are errors", () => {
		const dot = fixtures.find((fixture) => fixture.id === "dot");
		assert.throws(
			() => renderIcon("zero-ratio", dot.svg, 0),
			/icon `zero-ratio` has zero size/,
		);
		assert.throws(
			() =>
				buildSpriteSheet(
					[
						{ id: "same", svg: dot.svg },
						{ id: "same", svg: dot.svg },
					],
					1,
				),
			/duplicate icon id `same`/,
		);
		assert.throws(() => buildSpriteSheet([], 1), /no icons given/);
	});
});
