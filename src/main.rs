use std::io::{stdout, Write};
use std::path::Path;
use std::{env, fs};

use anyhow::anyhow;
use strconf::Config;

fn main() -> anyhow::Result<()> {
	let Some(input_path) = env::args_os().nth(1) else {
		return Err(anyhow!("No file path given"));
	};

	let input_path: &Path = input_path.as_ref();

	let ext = input_path
		.extension()
		.and_then(|x| x.to_str())
		.map(|x| x.to_ascii_lowercase().into_bytes())
		.unwrap_or_default();

	let file = || fs::read_to_string(input_path);

	let parsed: Config = if ext == b"json" {
		serde_json::from_str(&file()?)?
	} else if ext == b"yaml" || ext == b"yml" {
		serde_yaml::from_str(&file()?)?
	} else if ext == b"toml" {
		toml::from_str(&file()?)?
	} else {
		return Err(anyhow!("File name should end with supported format"));
	};

	let unparsed = serde_json::to_string_pretty(&parsed)?;
	stdout().write_all(unparsed.as_bytes())?;

	Ok(())
}
