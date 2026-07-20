import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

import * as browserApi from "@kartore/spritore";
import {
	renderIcon as renderBrowserIcon,
} from "@kartore/spritore";
import * as nodeApi from "@kartore/spritore/node";
import {
	buildSpriteSheet,
	renderIcon,
	type IconSource,
} from "@kartore/spritore/node";

const fixtureDirectory = new URL(
	"../../crates/spritore-core/tests/fixtures/",
	import.meta.url,
);
const goldenDirectory = new URL(
	"../../crates/spritore-core/tests/golden/",
	import.meta.url,
);
const wasmBytes = await readFile(
	new URL("../pkg/spritore_wasm_bg.wasm", import.meta.url),
);
const fixtureNames = ["dot", "pin", "grad", "line", "dot-copy", "spectrum"];
const fixtures: IconSource[] = await Promise.all(
	fixtureNames.map(async (id) => ({
		id,
		svg: await readFile(new URL(`${id}.svg`, fixtureDirectory), "utf8"),
	})),
);

test("JavaScript API", async (context) => {
	await context.test("initialization is automatic and not exported", async () => {
		assert.equal("init" in browserApi, false);
		assert.equal("init" in nodeApi, false);

		const dot = fixture("dot");
		const rendered = await renderBrowserIcon(dot.id, dot.svg, 1, {
			wasm: wasmBytes,
		});
		assert.equal(rendered.id, "dot");
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
				const sheet = await buildSpriteSheet(
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

	await context.test("public results are frozen plain objects", async () => {
		const dot = fixture("dot");
		const rendered = await renderIcon(dot.id, dot.svg, 1);

		assert.equal(Object.getPrototypeOf(rendered), Object.prototype);
		assert.ok(Object.isFrozen(rendered));
		assert.equal("renderIcon" in rendered, false);
		assert.equal("dispose" in rendered, false);
		assert.equal(Symbol.dispose in rendered, false);
		assert.equal(rendered.id, "dot");
		assert.equal(rendered.width, 15);
		assert.equal(rendered.height, 15);
		assert.equal(rendered.pixels.length, 15 * 15 * 4);
		assert.ok(rendered.pixels instanceof Uint8Array);

		const sheet = await buildSpriteSheet([dot], 1, { fast: true });
		assert.equal(Object.getPrototypeOf(sheet), Object.prototype);
		assert.equal(Object.getPrototypeOf(sheet.index), Object.prototype);
		assert.ok(Object.isFrozen(sheet));
		assert.ok(Object.isFrozen(sheet.index));
		assert.ok(Object.values(sheet.index).every(Object.isFrozen));
		assert.equal("buildSpriteSheet" in sheet, false);
	});

	await context.test("index keys remain own properties on a plain object", async () => {
		const dot = fixture("dot");
		const sheet = await buildSpriteSheet(
			[{ id: "__proto__", svg: dot.svg }],
			1,
			{ fast: true },
		);
		assert.ok(Object.hasOwn(sheet.index, "__proto__"));
		assert.equal(Object.getPrototypeOf(sheet.index), Object.prototype);
		assert.ok(Object.isFrozen(sheet.index.__proto__));
	});

	await context.test("invalid SVG errors include the icon id", async () => {
		await assert.rejects(
			renderIcon("broken-icon", "not svg", 1),
			(error) => error instanceof Error && error.message.includes("broken-icon"),
		);
	});

	await context.test("zero ratio, duplicate, and empty inputs are errors", async () => {
		const dot = fixture("dot");
		await assert.rejects(
			renderIcon("zero-ratio", dot.svg, 0),
			/icon `zero-ratio` has zero size/,
		);
		await assert.rejects(
			buildSpriteSheet(
				[
					{ id: "same", svg: dot.svg },
					{ id: "same", svg: dot.svg },
				],
				1,
			),
			/duplicate icon id `same`/,
		);
		await assert.rejects(buildSpriteSheet([], 1), /no icons given/);
	});
});

function fixture(id: string): IconSource {
	const result = fixtures.find((candidate) => candidate.id === id);
	assert.ok(result);
	return result;
}
