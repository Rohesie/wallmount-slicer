use anyhow::bail;
use yaml_rust::YamlLoader;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use anyhow::Result;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct PrefHolder {
	pub x_step: u32,
	pub y_step: u32,
	pub north_start_x: u32,
	pub north_start_y: u32,
	pub east_start_x: u32,
	pub east_start_y: u32,
	pub south_start_x: u32,
	pub south_start_y: u32,
	pub west_start_x: u32,
	pub west_start_y: u32,
}

pub fn load_configs(caller_path: String) -> Result<PrefHolder> {
	let config_path;
	let last_slash = caller_path.rfind(|c| c == '/' || c == '\\');
	if last_slash != None {
		config_path = caller_path[..last_slash.unwrap()].to_string();
	} else {
		config_path = ".".to_string();
	};
	let path = Path::new(&config_path).join("config.yaml");
	let mut file = File::open(path)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	let docs = YamlLoader::load_from_str(&contents).unwrap();
	let doc = &docs[0];

	let x_step = read_necessary_u32_config(&doc, "x_step")?;
	let y_step = read_necessary_u32_config(&doc, "y_step")?;
	let north_start_x = read_necessary_u32_config(&doc, "north_start_x")?;
	let north_start_y = read_necessary_u32_config(&doc, "north_start_y")?;
	let east_start_x = read_necessary_u32_config(&doc, "east_start_x")?;
	let east_start_y = read_necessary_u32_config(&doc, "east_start_y")?;
	let south_start_x = read_necessary_u32_config(&doc, "south_start_x")?;
	let south_start_y = read_necessary_u32_config(&doc, "south_start_y")?;
	let west_start_x = read_necessary_u32_config(&doc, "west_start_x")?;
	let west_start_y = read_necessary_u32_config(&doc, "west_start_y")?;

	return Ok(PrefHolder {
		x_step,
		y_step,
		north_start_x,
		north_start_y,
		east_start_x,
		east_start_y,
		south_start_x,
		south_start_y,
		west_start_x,
		west_start_y,
	})
}

pub fn read_necessary_u32_config(source: &yaml_rust::yaml::Yaml, index: &str) -> Result<u32> {
	let config = &source[index];
	if config.is_badvalue() {
		bail!("Undefined value for {}. This is a necessary config. Please check config.yaml in the examples folder for documentation.", index);
	};
	return match source[index].as_i64() {
		Some(thing) => Ok(thing as u32),
		None => bail!(
			"Unlawful value for {}, not a proper number: ({:?})",
			index,
			source[index]
		),
	};
}
