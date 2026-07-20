#!/usr/bin/env node

import { readFile, readdir, stat, writeFile, mkdir } from "node:fs/promises";
import { basename, extname, join } from "node:path";
import {
	parseArgs,
	type ParseArgsOptionsConfig,
} from "node:util";

import {
	buildSpriteSheet,
	renderIcon,
	type IconSource,
} from "./node.js";

const USAGE =
	"Usage: spritore build <svg-dir> -o <out-dir> " +
	"[--name sprite] [--ratio 1,2] [--fast] [--skip-invalid]";

class UsageError extends Error {}

interface Source extends IconSource {
	readonly fileName: string;
}

interface Output {
	readonly ratio: number;
	readonly png: Uint8Array;
	readonly indexJson: string;
}

try {
	await main();
} catch (error) {
	console.error(`error: ${errorMessage(error)}`);
	if (error instanceof UsageError) {
		console.error(`\n${USAGE}`);
	}
	process.exitCode = 1;
}

async function main(): Promise<void> {
	const { positionals, values } = parseArguments();
	if (values.help === true) {
		console.log(USAGE);
		return;
	}
	if (positionals.length !== 2 || positionals[0] !== "build") {
		throw new UsageError("expected the `build` subcommand and one SVG directory");
	}
	const inputDirectory = positionals[1];
	if (inputDirectory === undefined) {
		throw new UsageError("expected one SVG directory");
	}
	const outputDirectory = values.output;
	if (typeof outputDirectory !== "string") {
		throw new UsageError("the `-o, --output <out-dir>` option is required");
	}

	await requireInputDirectory(inputDirectory);
	const name = typeof values.name === "string" ? values.name : "sprite";
	validateOutputName(name);
	const ratios = parseRatios(
		typeof values.ratio === "string" ? values.ratio : "1,2",
	);

	const sources = await readSources(inputDirectory);
	const validSources = await validateSources(
		sources,
		values["skip-invalid"] === true,
	);
	const icons = validSources.map(({ id, svg }) => ({ id, svg }));
	const outputs: Output[] = [];
	for (const ratio of ratios) {
		const sheet = await buildSpriteSheet(
			icons,
			ratio,
			values.fast === true ? { fast: true } : undefined,
		);
		outputs.push({
			ratio,
			png: sheet.png,
			indexJson: sheet.indexJson,
		});
	}

	await mkdir(outputDirectory, { recursive: true });
	for (const output of outputs) {
		const suffix = output.ratio === 1 ? "" : `@${output.ratio}x`;
		await writeFile(
			join(outputDirectory, `${name}${suffix}.png`),
			output.png,
		);
		await writeFile(
			join(outputDirectory, `${name}${suffix}.json`),
			output.indexJson,
		);
	}
}

function parseArguments() {
	const options = {
		output: { type: "string", short: "o" },
		name: { type: "string" },
		ratio: { type: "string" },
		fast: { type: "boolean" },
		"skip-invalid": { type: "boolean" },
		help: { type: "boolean", short: "h" },
	} satisfies ParseArgsOptionsConfig;

	try {
		return parseArgs({
			allowPositionals: true,
			options,
			strict: true,
		});
	} catch (error) {
		throw new UsageError(errorMessage(error));
	}
}

async function requireInputDirectory(directory: string): Promise<void> {
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

function validateOutputName(name: string): void {
	if (name.length === 0) {
		throw new UsageError("--name must not be empty");
	}
	if (name.includes("/") || name.includes("\\")) {
		throw new UsageError("--name must not contain path separators");
	}
}

function parseRatios(value: string): number[] {
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

function invalidRatio(value: string): UsageError {
	return new UsageError(
		`invalid --ratio \`${value}\`: expected comma-separated integers from 1 to 255`,
	);
}

async function readSources(directory: string): Promise<Source[]> {
	const entries = (await readdir(directory, { withFileTypes: true }))
		.filter((entry) => entry.isFile() && extname(entry.name) === ".svg")
		.sort((left, right) =>
			Buffer.compare(Buffer.from(left.name), Buffer.from(right.name)),
		);
	if (entries.length === 0) {
		throw new Error(`no SVG files found in ${directory}`);
	}

	const ids = new Map<string, string>();
	const sources: Source[] = [];
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

function sanitizeId(value: string): string {
	return value.replace(/[^a-zA-Z0-9_-]/gu, "-");
}

async function validateSources(
	sources: readonly Source[],
	skipInvalid: boolean,
): Promise<Source[]> {
	const valid: Source[] = [];
	for (const source of sources) {
		try {
			await renderIcon(source.id, source.svg, 1);
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

function errorMessage(error: unknown): string {
	return error instanceof Error ? error.message : String(error);
}
