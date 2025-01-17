use std::{fs, path::PathBuf, sync::LazyLock};

use full_moon::LuaVersion;
use insta::{assert_debug_snapshot, glob, Settings};
use selene_lib::{lints::Severity, Checker};

static CHECKER: LazyLock<Checker<toml::value::Value>> = LazyLock::new(|| {
	let working_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

	let standard_library_path = working_dir.join("roblox.yml");
	let standard_library_file = fs::read_to_string(standard_library_path).expect("Unable to read the standard library");
	let standard_library = serde_yml::from_str(&standard_library_file).expect("Unable to parse the standard library");

	let config_path = working_dir.join("selene.toml");
	let config_file = fs::read_to_string(config_path).expect("Unable to read selene.toml");
	let config = toml::from_str(&config_file).expect("Unable to decode selene.toml");

	Checker::new(config, standard_library).expect("Unable to initialise selene!")
});

pub fn run_selene_test(input: &str, no_warnings: bool, insta_settings: Settings) {
	let generated_code = zap::run(input, no_warnings);

	let client_ast = full_moon::parse_fallible(&generated_code.code.unwrap().client.code, LuaVersion::luau())
		.into_result()
		.expect("Unable to parse code!");
	let client_diagnostics = CHECKER.test_on(&client_ast);

	insta_settings.bind(|| {
		assert_debug_snapshot!(client_diagnostics);
		assert!(client_diagnostics
			.iter()
			.all(|diagnostic| diagnostic.severity != Severity::Error));
	});
}

#[test]
fn test_lints() {
	glob!(env!("CARGO_MANIFEST_DIR"), "tests/files/*.zap", |path| {
		let input = fs::read_to_string(path).unwrap();

		let mut insta_settings = Settings::new();
		insta_settings.set_prepend_module_to_snapshot(false);
		insta_settings.set_sort_maps(true);
		insta_settings.set_input_file(path);
		insta_settings.set_snapshot_suffix(path.file_stem().unwrap().to_string_lossy());

		run_selene_test(&input, true, insta_settings)
	});
}
