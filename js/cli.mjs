#!/usr/bin/env node

import { readFile, readdir, stat, writeFile, mkdir } from "node:fs/promises";
import { basename, extname, join } from "node:path";
import { parseArgs } from "node:util";

import { buildSpriteSheet, init, renderIcon } from "./node.mjs";

const USAGE =
	"Usage: spritore build <svg-dir> -o <out-dir> " +
	"[--name sprite] [--ratio 1,2] [--fast] [--skip-invalid]";

class UsageError extends Error {}

try {
	await main();
} catch (error) {
	console.error(`error: ${errorMessage(error)}`);
	if (error instanceof UsageError) {
		console.error(`\n${USAGE}`);
	}
	process.exitCode = 1;
}

async function main() {
	const { positionals, values } = parseArguments();
	if (values.help) {
		console.log(USAGE);
		return;
	}
	if (positionals.length !== 2 || positionals[0] !== "build") {
		throw new UsageError("expected the `build` subcommand and one SVG directory");
	}
	if (values.output === undefined) {
		throw new UsageError("the `-o, --output <out-dir>` option is required");
	}

	const inputDirectory = positionals[1];
	await requireInputDirectory(inputDirectory);
	const name = values.name ?? "sprite";
	validateOutputName(name);
	const ratios = parseRatios(values.ratio ?? "1,2");

	await init();
	const sources = await readSources(inputDirectory);
	const validSources = validateSources(sources, values["skip-invalid"] ?? false);
	const outputs = ratios.map((ratio) => {
		const sheet = buildSpriteSheet(
			validSources.map(({ id, svg }) => ({ id, svg })),
			ratio,
			values.fast ? { fast: true } : undefined,
		);
		return { ratio, png: sheet.png, indexJson: sheet.indexJson };
	});

	await mkdir(values.output, { recursive: true });
	for (const output of outputs) {
		const suffix = output.ratio === 1 ? "" : `@${output.ratio}x`;
		await writeFile(join(values.output, `${name}${suffix}.png`), output.png);
		await writeFile(
			join(values.output, `${name}${suffix}.json`),
			output.indexJson,
		);
	}
}

function parseArguments() {
	try {
		return parseArgs({
			allowPositionals: true,
			options: {
				output: { type: "string", short: "o" },
				name: { type: "string" },
				ratio: { type: "string" },
				fast: { type: "boolean" },
				"skip-invalid": { type: "boolean" },
				help: { type: "boolean", short: "h" },
			},
			strict: true,
		});
	} catch (error) {
		throw new UsageError(errorMessage(error));
	}
}

async function requireInputDirectory(directory) {
	let metadata;
	try {
		metadata = await stat(directory);
	} catch {
		throw new UsageError(`input directory does not exist: ${directory}`);
	}
	if (!metadata.isDirectory()) {
		throw new UsageError(`input path is not a directory: ${directory}`);
	}
}

function validateOutputName(name) {
	if (name.length === 0) {
		throw new UsageError("--name must not be empty");
	}
	if (name.includes("/") || name.includes("\\")) {
		throw new UsageError("--name must not contain path separators");
	}
}

function parseRatios(value) {
	const parts = value.split(",");
	if (
		parts.length === 0 ||
		parts.some((part) => !/^[0-9]+$/.test(part))
	) {
		throw invalidRatio(value);
	}

	const ratios = parts.map(Number);
	if (ratios.some((ratio) => ratio < 1 || ratio > 255)) {
		throw invalidRatio(value);
	}
	if (new Set(ratios).size !== ratios.length) {
		throw new UsageError("--ratio values must be unique");
	}
	return ratios;
}

function invalidRatio(value) {
	return new UsageError(
		`invalid --ratio \`${value}\`: expected comma-separated integers from 1 to 255`,
	);
}

async function readSources(directory) {
	const entries = (await readdir(directory, { withFileTypes: true }))
		.filter((entry) => entry.isFile() && extname(entry.name) === ".svg")
		.sort((left, right) =>
			Buffer.compare(Buffer.from(left.name), Buffer.from(right.name)),
		);
	if (entries.length === 0) {
		throw new Error(`no SVG files found in ${directory}`);
	}

	const ids = new Map();
	const sources = [];
	for (const entry of entries) {
		const stem = basename(entry.name, ".svg");
		const id = sanitizeId(stem);
		const existing = ids.get(id);
		if (existing !== undefined) {
			throw new Error(
				`icon id collision \`${id}\` after sanitizing ` +
					`\`${existing}\` and \`${entry.name}\``,
			);
		}
		ids.set(id, entry.name);
		sources.push({
			fileName: entry.name,
			id,
			svg: await readFile(join(directory, entry.name), "utf8"),
		});
	}
	return sources;
}

function sanitizeId(value) {
	return value.replace(/[^a-zA-Z0-9_-]/gu, "-");
}

function validateSources(sources, skipInvalid) {
	const valid = [];
	for (const source of sources) {
		try {
			renderIcon(source.id, source.svg, 1);
			valid.push(source);
		} catch (error) {
			const message = errorMessage(error);
			if (skipInvalid && message.startsWith("invalid SVG for icon")) {
				console.error(`skipping \`${source.fileName}\`: ${message}`);
				continue;
			}
			throw new Error(`failed to render \`${source.fileName}\`: ${message}`);
		}
	}
	if (valid.length === 0) {
		throw new Error("no valid SVG files found");
	}
	return valid;
}

function errorMessage(error) {
	return error instanceof Error ? error.message : String(error);
}
