import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
	copyFile,
	mkdir,
	mkdtemp,
	readFile,
	readdir,
	rm,
	writeFile,
} from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const cliPath = fileURLToPath(new URL("../cli.mjs", import.meta.url));
const fixtureDirectory = fileURLToPath(
	new URL("../../crates/spritore-core/tests/fixtures/", import.meta.url),
);
const goldenDirectory = fileURLToPath(
	new URL("../../crates/spritore-core/tests/golden/", import.meta.url),
);

test("Node CLI", async (context) => {
	const temporaryRoot = await mkdtemp(join(tmpdir(), "spritore-node-cli-"));
	context.after(() => rm(temporaryRoot, { recursive: true, force: true }));

	await context.test("matches zopfli and fast golden files", async () => {
		const zopfliOutput = join(temporaryRoot, "zopfli");
		const zopfli = runCli([
			"build",
			fixtureDirectory,
			"-o",
			zopfliOutput,
		]);
		assert.equal(zopfli.status, 0, zopfli.stderr);
		await assertGoldenOutput(zopfliOutput, false);

		const fastOutput = join(temporaryRoot, "fast");
		const fast = runCli([
			"build",
			fixtureDirectory,
			"-o",
			fastOutput,
			"--fast",
		]);
		assert.equal(fast.status, 0, fast.stderr);
		await assertGoldenOutput(fastOutput, true);
	});

	await context.test("invalid SVGs fail without partial output and can be skipped", async () => {
		const input = join(temporaryRoot, "invalid-input");
		await mkdir(input);
		await copyFile(join(fixtureDirectory, "dot.svg"), join(input, "dot.svg"));
		await writeFile(join(input, "broken.svg"), "not svg");

		const failedOutput = join(temporaryRoot, "invalid-output");
		const failed = runCli(["build", input, "-o", failedOutput]);
		assert.equal(failed.status, 1);
		assert.match(failed.stderr, /broken\.svg/);
		assert.equal(existsSync(failedOutput), false);

		const skippedOutput = join(temporaryRoot, "skipped-output");
		const skipped = runCli([
			"build",
			input,
			"-o",
			skippedOutput,
			"--skip-invalid",
		]);
		assert.equal(skipped.status, 0, skipped.stderr);
		assert.match(skipped.stderr, /skipping `broken\.svg`/);
		const index = JSON.parse(
			await readFile(join(skippedOutput, "sprite.json"), "utf8"),
		);
		assert.deepEqual(Object.keys(index), ["dot"]);
	});

	await context.test("sanitized ID collisions are errors", async () => {
		const input = join(temporaryRoot, "collision-input");
		await mkdir(input);
		await copyFile(join(fixtureDirectory, "dot.svg"), join(input, "a b.svg"));
		await copyFile(join(fixtureDirectory, "pin.svg"), join(input, "a-b.svg"));

		const output = join(temporaryRoot, "collision-output");
		const result = runCli(["build", input, "-o", output]);
		assert.equal(result.status, 1);
		assert.match(result.stderr, /icon id collision `a-b`/);
		assert.equal(existsSync(output), false);
	});

	await context.test("usage errors include usage and exit one", () => {
		const output = join(temporaryRoot, "missing-output");
		const result = runCli([
			"build",
			join(temporaryRoot, "does-not-exist"),
			"-o",
			output,
		]);
		assert.equal(result.status, 1);
		assert.match(result.stderr, /Usage: spritore build/);
		assert.equal(existsSync(output), false);
	});
});

function runCli(arguments_) {
	return spawnSync(process.execPath, [cliPath, ...arguments_], {
		encoding: "utf8",
	});
}

async function assertGoldenOutput(outputDirectory, fast) {
	assert.deepEqual((await readdir(outputDirectory)).sort(), [
		"sprite.json",
		"sprite.png",
		"sprite@2x.json",
		"sprite@2x.png",
	]);

	for (const ratio of [1, 2]) {
		const suffix = ratio === 1 ? "" : "@2x";
		const kind = fast ? ".fast" : "";
		const [actualPng, expectedPng, actualJson, expectedJson] = await Promise.all([
			readFile(join(outputDirectory, `sprite${suffix}.png`)),
			readFile(join(goldenDirectory, `sprite${suffix}${kind}.png`)),
			readFile(join(outputDirectory, `sprite${suffix}.json`)),
			readFile(join(goldenDirectory, `sprite${suffix}.json`)),
		]);
		assert.equal(Buffer.compare(actualPng, expectedPng), 0);
		assert.equal(Buffer.compare(actualJson, expectedJson), 0);
	}
}
