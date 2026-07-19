#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use spritore::{BuildOptions, Error, build_sprite_sheet, index_to_json, render_icon};

const USAGE: &str = concat!(
	"Usage: spritore build <svg-dir> -o <out-dir> ",
	"[--name sprite] [--ratio 1,2] [--fast] [--skip-invalid]",
);

#[derive(Parser)]
#[command(name = "spritore", about = "MapLibre sprite generator")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Builds PNG and JSON sprite assets from a directory of SVG icons.
	Build(BuildArgs),
}

#[derive(Args)]
struct BuildArgs {
	/// Directory containing .svg files.
	#[arg(value_name = "SVG_DIR")]
	svg_dir: PathBuf,
	/// Directory to receive the generated assets.
	#[arg(short = 'o', long = "output", value_name = "OUT_DIR")]
	output_dir: PathBuf,
	/// Base name for generated files.
	#[arg(long, default_value = "sprite")]
	name: String,
	/// Comma-separated pixel ratios.
	#[arg(long, default_value = "1,2")]
	ratio: String,
	/// Uses miniz instead of Zopfli compression.
	#[arg(long)]
	fast: bool,
	/// Skips SVG parse errors instead of failing the build.
	#[arg(long)]
	skip_invalid: bool,
}

struct CliFailure {
	message: String,
	show_usage: bool,
}

impl CliFailure {
	fn runtime(message: impl Into<String>) -> Self {
		Self {
			message: message.into(),
			show_usage: false,
		}
	}

	fn usage(message: impl Into<String>) -> Self {
		Self {
			message: message.into(),
			show_usage: true,
		}
	}
}

struct IconSource {
	file_name: String,
	id: String,
	svg: String,
}

struct BuiltSheet {
	ratio: u8,
	png: Vec<u8>,
	index_json: String,
}

fn main() -> ExitCode {
	let cli = match Cli::try_parse() {
		Ok(cli) => cli,
		Err(error) => {
			let exit_code = error.exit_code();
			if let Err(print_error) = error.print() {
				eprintln!("error: failed to print CLI help: {print_error}");
			}
			return if exit_code == 0 {
				ExitCode::SUCCESS
			} else {
				ExitCode::FAILURE
			};
		}
	};

	match run(cli) {
		Ok(()) => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("error: {}", error.message);
			if error.show_usage {
				eprintln!("\n{USAGE}");
			}
			ExitCode::FAILURE
		}
	}
}

fn run(cli: Cli) -> Result<(), CliFailure> {
	match cli.command {
		Command::Build(arguments) => build(arguments),
	}
}

fn build(arguments: BuildArgs) -> Result<(), CliFailure> {
	if !arguments.svg_dir.exists() {
		return Err(CliFailure::usage(format!(
			"input directory does not exist: {}",
			arguments.svg_dir.display(),
		)));
	}
	if !arguments.svg_dir.is_dir() {
		return Err(CliFailure::usage(format!(
			"input path is not a directory: {}",
			arguments.svg_dir.display(),
		)));
	}
	validate_output_name(&arguments.name)?;
	let ratios = parse_ratios(&arguments.ratio)?;
	let sources = read_sources(&arguments.svg_dir)?;
	let sources = validate_sources(sources, arguments.skip_invalid)?;
	let mut outputs = Vec::with_capacity(ratios.len());

	for ratio in ratios {
		let rendered = sources
			.iter()
			.map(|source| render_icon(&source.id, &source.svg, ratio))
			.collect::<Result<Vec<_>, _>>()
			.map_err(|error| CliFailure::runtime(error.to_string()))?;
		let sheet = build_sprite_sheet(
			&rendered,
			ratio,
			BuildOptions {
				fast: arguments.fast,
			},
		)
		.map_err(|error| CliFailure::runtime(error.to_string()))?;
		outputs.push(BuiltSheet {
			ratio,
			index_json: index_to_json(&sheet.index),
			png: sheet.png,
		});
	}

	write_outputs(&arguments.output_dir, &arguments.name, &outputs)
}

fn validate_output_name(name: &str) -> Result<(), CliFailure> {
	if name.is_empty() {
		return Err(CliFailure::usage("--name must not be empty"));
	}
	if name.contains('/') || name.contains('\\') {
		return Err(CliFailure::usage("--name must not contain path separators"));
	}
	Ok(())
}

fn parse_ratios(value: &str) -> Result<Vec<u8>, CliFailure> {
	let mut ratios = Vec::new();
	for part in value.split(',') {
		if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
			return Err(invalid_ratio(value));
		}
		let ratio = part.parse::<u16>().map_err(|_| invalid_ratio(value))?;
		if !(1..=u16::from(u8::MAX)).contains(&ratio) {
			return Err(invalid_ratio(value));
		}
		let ratio = ratio as u8;
		if ratios.contains(&ratio) {
			return Err(CliFailure::usage("--ratio values must be unique"));
		}
		ratios.push(ratio);
	}
	Ok(ratios)
}

fn invalid_ratio(value: &str) -> CliFailure {
	CliFailure::usage(format!(
		"invalid --ratio `{value}`: expected comma-separated integers from 1 to 255",
	))
}

fn read_sources(directory: &Path) -> Result<Vec<IconSource>, CliFailure> {
	let entries = fs::read_dir(directory).map_err(|error| {
		CliFailure::runtime(format!(
			"failed to read input directory {}: {error}",
			directory.display(),
		))
	})?;
	let mut files = Vec::new();
	for entry in entries {
		let entry = entry.map_err(|error| {
			CliFailure::runtime(format!(
				"failed to read input directory {}: {error}",
				directory.display(),
			))
		})?;
		let file_type = entry.file_type().map_err(|error| {
			CliFailure::runtime(format!(
				"failed to inspect {}: {error}",
				entry.path().display(),
			))
		})?;
		let path = entry.path();
		if !file_type.is_file() || path.extension() != Some(OsStr::new("svg")) {
			continue;
		}
		let file_name = entry.file_name().into_string().map_err(|_| {
			CliFailure::runtime(format!(
				"SVG filename is not valid UTF-8: {}",
				path.display(),
			))
		})?;
		files.push((file_name, path));
	}
	files.sort_by(|left, right| left.0.cmp(&right.0));
	if files.is_empty() {
		return Err(CliFailure::runtime(format!(
			"no SVG files found in {}",
			directory.display(),
		)));
	}

	let mut ids = BTreeMap::new();
	let mut sources = Vec::with_capacity(files.len());
	for (file_name, path) in files {
		let stem = file_name
			.strip_suffix(".svg")
			.expect("the extension was checked above");
		let id = sanitize_id(stem);
		if let Some(existing) = ids.insert(id.clone(), file_name.clone()) {
			return Err(CliFailure::runtime(format!(
				"icon id collision `{id}` after sanitizing `{existing}` and `{file_name}`",
			)));
		}
		let svg = fs::read_to_string(&path).map_err(|error| {
			CliFailure::runtime(format!("failed to read {}: {error}", path.display()))
		})?;
		sources.push(IconSource { file_name, id, svg });
	}
	Ok(sources)
}

fn sanitize_id(value: &str) -> String {
	value
		.chars()
		.map(|character| {
			if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
				character
			} else {
				'-'
			}
		})
		.collect()
}

fn validate_sources(
	sources: Vec<IconSource>,
	skip_invalid: bool,
) -> Result<Vec<IconSource>, CliFailure> {
	let mut valid = Vec::with_capacity(sources.len());
	for source in sources {
		match render_icon(&source.id, &source.svg, 1) {
			Ok(_) => valid.push(source),
			Err(error) if skip_invalid && matches!(&error, Error::InvalidSvg { .. }) => {
				eprintln!("skipping `{}`: {error}", source.file_name);
			}
			Err(error) => {
				return Err(CliFailure::runtime(format!(
					"failed to render `{}`: {error}",
					source.file_name,
				)));
			}
		}
	}
	if valid.is_empty() {
		return Err(CliFailure::runtime("no valid SVG files found"));
	}
	Ok(valid)
}

fn write_outputs(directory: &Path, name: &str, outputs: &[BuiltSheet]) -> Result<(), CliFailure> {
	fs::create_dir_all(directory).map_err(|error| {
		CliFailure::runtime(format!(
			"failed to create output directory {}: {error}",
			directory.display(),
		))
	})?;
	for output in outputs {
		let suffix = if output.ratio == 1 {
			String::new()
		} else {
			format!("@{}x", output.ratio)
		};
		let png_path = directory.join(format!("{name}{suffix}.png"));
		fs::write(&png_path, &output.png).map_err(|error| {
			CliFailure::runtime(format!("failed to write {}: {error}", png_path.display()))
		})?;
		let json_path = directory.join(format!("{name}{suffix}.json"));
		fs::write(&json_path, output.index_json.as_bytes()).map_err(|error| {
			CliFailure::runtime(format!("failed to write {}: {error}", json_path.display()))
		})?;
	}
	Ok(())
}
