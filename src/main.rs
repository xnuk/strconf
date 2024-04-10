use std::io::{stdout, Write};
use std::{env, fs};

use anyhow::anyhow;
use strconf::Config;

fn main() -> anyhow::Result<()> {
	let Some(input_path) = env::args_os().nth(1) else {
		return Err(anyhow!("No file path given"));
	};

	let file = fs::read_to_string(input_path)?;
	let parsed: Config = toml::from_str(&file)?;
	let unparsed = toml::to_string_pretty(&parsed)?;
	stdout().write_all(unparsed.as_bytes())?;

	Ok(())
}
