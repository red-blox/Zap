use crate::config::Parameter;

pub mod luau;
pub mod tooling;
pub mod typescript;

pub fn get_unnamed_values(prefix: &str, count: usize) -> Vec<String> {
	(0..count)
		.map(|i| {
			if i > 0 {
				format!("{prefix}{}", i + 1)
			} else {
				prefix.to_string()
			}
		})
		.collect()
}

pub fn get_named_values(default_prefix: &str, parameters: &[Parameter]) -> Vec<String> {
	parameters
		.iter()
		.enumerate()
		.map(|(i, parameter)| match parameter.name {
			Some(name) => name.to_string(),
			None => {
				if i > 0 {
					format!("{default_prefix}{}", i + 1)
				} else {
					default_prefix.to_string()
				}
			}
		})
		.collect()
}
