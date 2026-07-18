use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TempDirectory(PathBuf);

impl TempDirectory {
	fn new(label: &str) -> Self {
		let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
		let path = std::env::temp_dir().join(format!(
			"spritore-cli-{label}-{}-{sequence}",
			std::process::id(),
		));
		fs::create_dir_all(&path).unwrap();
		Self(path)
	}

	fn path(&self) -> &Path {
		&self.0
	}
}

impl Drop for TempDirectory {
	fn drop(&mut self) {
		let _ = fs::remove_dir_all(&self.0);
	}
}

fn fixture_directory() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("../spritore-core/tests/fixtures")
}

fn golden_directory() -> PathBuf {
	Path::new(env!("CARGO_MANIFEST_DIR")).join("../spritore-core/tests/golden")
}

fn run_cli(arguments: &[&str]) -> Output {
	Command::new(env!("CARGO_BIN_EXE_spritore"))
		.args(arguments)
		.output()
		.unwrap()
}

fn assert_success(output: &Output) {
	assert!(
		output.status.success(),
		"CLI failed:\nstdout:\n{}\nstderr:\n{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr),
	);
}

fn assert_golden_output(directory: &Path, fast: bool) {
	let mut names: Vec<_> = fs::read_dir(directory)
		.unwrap()
		.map(|entry| entry.unwrap().file_name().into_string().unwrap())
		.collect();
	names.sort();
	assert_eq!(
		names,
		[
			"sprite.json",
			"sprite.png",
			"sprite@2x.json",
			"sprite@2x.png",
		],
	);

	for ratio in [1, 2] {
		let suffix = if ratio == 1 { "" } else { "@2x" };
		let kind = if fast { ".fast" } else { "" };
		assert_eq!(
			fs::read(directory.join(format!("sprite{suffix}.png"))).unwrap(),
			fs::read(golden_directory().join(format!("sprite{suffix}{kind}.png"))).unwrap(),
		);
		assert_eq!(
			fs::read(directory.join(format!("sprite{suffix}.json"))).unwrap(),
			fs::read(golden_directory().join(format!("sprite{suffix}.json"))).unwrap(),
		);
	}
}

#[test]
fn matches_zopfli_and_fast_golden_files() {
	let zopfli = TempDirectory::new("zopfli");
	let fixtures = fixture_directory();
	let output = run_cli(&[
		"build",
		fixtures.to_str().unwrap(),
		"-o",
		zopfli.path().to_str().unwrap(),
	]);
	assert_success(&output);
	assert_golden_output(zopfli.path(), false);

	let fast = TempDirectory::new("fast");
	let output = run_cli(&[
		"build",
		fixtures.to_str().unwrap(),
		"-o",
		fast.path().to_str().unwrap(),
		"--fast",
	]);
	assert_success(&output);
	assert_golden_output(fast.path(), true);
}

#[test]
fn invalid_svgs_fail_without_output_and_can_be_skipped() {
	let temporary = TempDirectory::new("invalid");
	let input = temporary.path().join("input");
	fs::create_dir(&input).unwrap();
	fs::copy(fixture_directory().join("dot.svg"), input.join("dot.svg")).unwrap();
	fs::write(input.join("broken.svg"), "not svg").unwrap();

	let failed_output = temporary.path().join("failed-output");
	let result = run_cli(&[
		"build",
		input.to_str().unwrap(),
		"-o",
		failed_output.to_str().unwrap(),
	]);
	assert_eq!(result.status.code(), Some(1));
	assert!(String::from_utf8_lossy(&result.stderr).contains("broken.svg"));
	assert!(!failed_output.exists());

	let skipped_output = temporary.path().join("skipped-output");
	let result = run_cli(&[
		"build",
		input.to_str().unwrap(),
		"-o",
		skipped_output.to_str().unwrap(),
		"--skip-invalid",
	]);
	assert_success(&result);
	assert!(String::from_utf8_lossy(&result.stderr).contains("skipping `broken.svg`"));
	let index = fs::read_to_string(skipped_output.join("sprite.json")).unwrap();
	assert!(index.contains("\"dot\""));
	assert!(!index.contains("\"broken\""));
}

#[test]
fn sanitized_id_collisions_are_errors() {
	let temporary = TempDirectory::new("collision");
	let input = temporary.path().join("input");
	fs::create_dir(&input).unwrap();
	fs::copy(fixture_directory().join("dot.svg"), input.join("a b.svg")).unwrap();
	fs::copy(fixture_directory().join("pin.svg"), input.join("a-b.svg")).unwrap();

	let output = temporary.path().join("output");
	let result = run_cli(&[
		"build",
		input.to_str().unwrap(),
		"-o",
		output.to_str().unwrap(),
	]);
	assert_eq!(result.status.code(), Some(1));
	assert!(String::from_utf8_lossy(&result.stderr).contains("icon id collision `a-b`"));
	assert!(!output.exists());
}

#[test]
fn missing_input_directory_is_a_usage_error() {
	let temporary = TempDirectory::new("usage");
	let missing = temporary.path().join("does-not-exist");
	let output = temporary.path().join("output");
	let result = run_cli(&[
		"build",
		missing.to_str().unwrap(),
		"-o",
		output.to_str().unwrap(),
	]);
	assert_eq!(result.status.code(), Some(1));
	assert!(String::from_utf8_lossy(&result.stderr).contains("Usage: spritore build"));
	assert!(!output.exists());
}
