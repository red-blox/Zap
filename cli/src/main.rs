use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use codespan_reporting::{
	files::SimpleFile,
	term::{
		self,
		termcolor::{ColorChoice, StandardStream},
	},
};
use zap::run;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(default_value = "net.zap")]
	config: Option<PathBuf>,
}

fn main() -> Result<()> {
	let args = Args::parse();

	let config_path = args.config.unwrap();

	let config = std::fs::read_to_string(&config_path)?;

	let ret = run(config.as_str());

	let code = ret.code;
	let diagnostics = ret.diagnostics;

	if let Some(code) = code {
		let server_path = code.server.path.unwrap_or(PathBuf::from("net/server.lua"));
		let client_path = code.client.path.unwrap_or(PathBuf::from("net/client.lua"));

		if let Some(parent) = server_path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		if let Some(parent) = client_path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		std::fs::write(server_path.clone(), code.server.code)?;
		std::fs::write(client_path.clone(), code.client.code)?;

		if let Some(defs) = code.server.defs {
			std::fs::write(server_path.with_extension("d.ts"), defs)?;
		}

		if let Some(defs) = code.client.defs {
			std::fs::write(client_path.with_extension("d.ts"), defs)?;
		}
	}

	if diagnostics.is_empty() {
		return Ok(());
	}

	let file = SimpleFile::new(config_path.to_str().unwrap(), config);

	let writer = StandardStream::stderr(ColorChoice::Always);
	let config_term = codespan_reporting::term::Config::default();

	for diagnostic in diagnostics {
		term::emit(&mut writer.lock(), &config_term, &file, &diagnostic)?;
	}

	Ok(())
}
