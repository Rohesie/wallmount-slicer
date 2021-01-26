mod config;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use anyhow::Result;
use dmi::icon;
use image::GenericImageView;

fn main() {
	let mut args: Vec<String> = env::args().collect();

	let self_path = args.remove(0);

	let prefs =	match config::load_configs(self_path.clone()) {
		Ok(thing) => thing,
		Err(e) => {
			println!("Failed to load configs: {:#?}\nAdd the config.yaml along with the essential configs for the program to work.", e);
			dont_disappear::any_key_to_continue::default();
			return;
		}
	};

	let max_x = [prefs.north_start_x, prefs.east_start_x, prefs.south_start_x, prefs.west_start_x].iter().max().unwrap() + prefs.x_step;
	let max_y = [prefs.north_start_y, prefs.east_start_y, prefs.south_start_y, prefs.west_start_y].iter().max().unwrap() + prefs.y_step;

	let mut states = vec![];

	for image_path_string in args.iter() {
		let path = Path::new(&image_path_string);
		let mut file;
		match File::open(&path) {
			Ok(f) => file = f,
			Err(e) => {
				println!("Wrong file path: {}\nError: {:#?}", image_path_string, e);
				dont_disappear::any_key_to_continue::default();
				return;
			}
		};
		let mut contents = Vec::new();
		if let Err(e) = file.read_to_end(&mut contents) {
			println!("Unable to read file: {}\nError: {:#?}", image_path_string, e);
			dont_disappear::any_key_to_continue::default();
			return;
		};
		let cursor = Cursor::new(contents);

		let img = match image::load(cursor, image::ImageFormat::Png) {
			Ok(x) => x,
			Err(e) => {
				println!("Failed to read file as png: {}\nError: {:#?}", image_path_string, e);
				dont_disappear::any_key_to_continue::default();
				return;
			}
		};

		let (width, height) = img.dimensions();
		if max_x > width || max_y > height {
			println!("Config and image mismatch: {}\nImage width / max config width: {} / {}\nImage height / max config height: {} / {}", image_path_string, width, max_x, height, max_y);
			continue
		}

		let mut formatted_file_name = image_path_string.clone();

		let dot_offset = image_path_string
			.find('.')
			.unwrap_or(image_path_string.len());
		formatted_file_name = formatted_file_name.drain(..dot_offset).collect(); //Here we remove everything after the dot. Whether .dmi or .png is the same for us.
		formatted_file_name = trim_path_before_last_slash(formatted_file_name);

		let building_return = build_icon_state(img, formatted_file_name, &prefs);
		let new_state = match building_return {
			Ok(x) => {
				println!("Icon state built successfully: {}", image_path_string);
				x
			},
			Err(e) => {
				println!("Error building icon state: {}\nError: {:#?}", image_path_string, e);
				continue
			},
		};
		states.push(new_state);
	}
	if states.len() > 0 {
		let new_icon = icon::Icon {
			version: Default::default(),
			width: prefs.x_step,
			height: prefs.y_step,
			states,
		};
		let dmi_path = Path::new("output.dmi");
		let mut file = match File::create(&dmi_path) {
			Ok(x) => x,
			Err(e) => {
				println!("Error creating path: {:#?}", e);
				dont_disappear::any_key_to_continue::default();
				return
			}
		};
		match new_icon.save(&mut file) {
			Ok(_x) => (),
			Err(e) => {
				println!("Error saving new icon: {:#?}", e);
				dont_disappear::any_key_to_continue::default();
				return
			}
		}
	};

	println!("Program finished.");
	dont_disappear::any_key_to_continue::default();
}

fn build_icon_state(
	input_image: image::DynamicImage,
	file_name: String,
	prefs: &config::PrefHolder,
) -> Result<icon::IconState> {

	let north_img = input_image.crop_imm(prefs.north_start_x, prefs.north_start_y, prefs.x_step, prefs.y_step);

	let east_img = input_image.crop_imm(prefs.east_start_x, prefs.east_start_y, prefs.x_step, prefs.y_step);

	let south_img = input_image.crop_imm(prefs.south_start_x, prefs.south_start_y, prefs.x_step, prefs.y_step);

	let west_img = input_image.crop_imm(prefs.west_start_x, prefs.west_start_y, prefs.x_step, prefs.y_step);

	let images = vec![south_img, north_img, east_img, west_img];

	Ok(icon::IconState {
		name: file_name,
		dirs: 4,
		images,
		..Default::default()
	})
}

///Takes everything that comes after the last slash (or backslash) in the string, discarding the rest.
pub fn trim_path_before_last_slash(mut text: String) -> String {
	if text.is_empty() {
		return text;
	};
	let is_slash = |c| c == '/' || c == '\\';
	let slash_offset = match text.rfind(is_slash) {
		Some(num) => num + 1,
		None => 0,
	};
	text.drain(..slash_offset);
	text
}
