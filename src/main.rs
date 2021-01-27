mod config;

use anyhow::bail;
use anyhow::Result;
use dmi::icon;
use image::gif::GifDecoder;
use image::AnimationDecoder;
use image::Frame;
use image::GenericImageView;
use std::env;
use std::fs::File;
use std::path::Path;

enum RawImg {
	Png(image::DynamicImage),
	Gif(Vec<Frame>),
}

impl RawImg {
	fn dimensions(&self) -> Result<(u32, u32)> {
		match self {
			RawImg::Png(img) => Ok(img.dimensions()),
			RawImg::Gif(frame_vec) => {
				if frame_vec.len() == 0 {
					bail!("dimensions() called for empty RawImg::Gif")
				};
				let sample_frame = &frame_vec[0];
				Ok(sample_frame.buffer().dimensions())
			}
		}
	}
}

fn main() {
	let mut args: Vec<String> = env::args().collect();

	let self_path = args.remove(0);

	if args.len() == 0 {
		println!("No images found to open.\nSolution: click and drag one or multiple images into the executable to feed it the requried information.");
		dont_disappear::any_key_to_continue::default();
		return;
	}

	let prefs = match config::load_configs(self_path.clone()) {
		Ok(thing) => thing,
		Err(e) => {
			println!("Failed to load configs: {:#?}\nSolution: Add the config.yaml along with the essential configs for the program to work.", e);
			dont_disappear::any_key_to_continue::default();
			return;
		}
	};

	let max_x = [
		prefs.north_start_x,
		prefs.east_start_x,
		prefs.south_start_x,
		prefs.west_start_x,
	]
	.iter()
	.max()
	.unwrap()
		+ prefs.x_step;
	let max_y = [
		prefs.north_start_y,
		prefs.east_start_y,
		prefs.south_start_y,
		prefs.west_start_y,
	]
	.iter()
	.max()
	.unwrap()
		+ prefs.y_step;

	let mut states = vec![];

	for image_path_string in args.iter() {
		let path = Path::new(&image_path_string);
		let raw_img = match image::ImageFormat::from_path(path) {
			Ok(image::ImageFormat::Png) => {
				let png = match image::open(path) {
					Ok(x) => x,
					Err(e) => {
						println!(
							"Failed to open file: {}\nError: {:#?}",
							image_path_string, e
						);
						dont_disappear::any_key_to_continue::default();
						return;
					}
				};
				RawImg::Png(png)
			}
			Ok(image::ImageFormat::Gif) => {
				let file = match File::open(&path) {
					Ok(f) => f,
					Err(e) => {
						println!("Wrong file path: {}\nError: {:#?}", image_path_string, e);
						continue;
					}
				};
				let decoder = match GifDecoder::new(file) {
					Ok(g) => g,
					Err(e) => {
						println!(
							"Unable to decode gif: {}\nError: {:#?}",
							image_path_string, e
						);
						continue;
					}
				};
				let frames = decoder.into_frames();
				let frames = match frames.collect_frames() {
					Ok(f) => f,
					Err(e) => {
						println!(
							"Unable to collect frames: {}\nError: {:#?}",
							image_path_string, e
						);
						continue;
					}
				};
				RawImg::Gif(frames)
			}
			_ => {
				println!(
					"Wrong format for path (only .png and .gif supported): {}",
					image_path_string
				);
				continue;
			}
		};

		let (width, height) = match raw_img.dimensions() {
			Ok((a, b)) => (a, b),
			Err(e) => {
				println!(
					"Failed ot check the image's dimensions {}\nError: {:#?}",
					image_path_string, e
				);
				continue;
			}
		};
		if max_x > width || max_y > height {
			println!("Config and image mismatch: {}\nImage width / max config width: {} / {}\nImage height / max config height: {} / {}", image_path_string, width, max_x, height, max_y);
			continue;
		}

		let mut formatted_file_name = image_path_string.clone();

		let dot_offset = image_path_string
			.find('.')
			.unwrap_or(image_path_string.len());
		formatted_file_name = formatted_file_name.drain(..dot_offset).collect(); //Here we remove everything after the dot. Whether .dmi or .png is the same for us.
		formatted_file_name = trim_path_before_last_slash(formatted_file_name);

		let building_return = build_icon_state(raw_img, formatted_file_name, &prefs);
		let new_state = match building_return {
			Ok(x) => {
				println!("Icon state built successfully: {}", image_path_string);
				x
			}
			Err(e) => {
				println!(
					"Error building icon state: {}\nError: {:#?}",
					image_path_string, e
				);
				continue;
			}
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
				return;
			}
		};
		match new_icon.save(&mut file) {
			Ok(_x) => (),
			Err(e) => {
				println!("Error saving new icon: {:#?}", e);
				dont_disappear::any_key_to_continue::default();
				return;
			}
		}
	};

	println!("Program finished.");
	dont_disappear::any_key_to_continue::default();
}

fn build_icon_state(
	raw_img: RawImg,
	file_name: String,
	prefs: &config::PrefHolder,
) -> Result<icon::IconState> {
	let (frames, images, delay) = match raw_img {
		RawImg::Png(image) => {
			let images = extract_four_dir_images(image, prefs);
			(1, images, None)
		}
		RawImg::Gif(frame_vec) => {
			if frame_vec.len() == 0 {
				bail!("Invalid gif image read, frame vector has zero elements.")
			};
			let frames_len = frame_vec.len() as u32;
			let mut images = vec![];
			let mut delay = vec![];
			let mut frame_iteration = 0;
			for frame in frame_vec {
				let (numerator, denominator) = frame.delay().numer_denom_ms();
				let frame_delay: f32 = (numerator as f32 / denominator as f32) / 100.0; //from ms to ds
				delay.push(frame_delay);

				let image = image::DynamicImage::ImageRgba8(frame.into_buffer());
				let mut frame_dir_images = extract_four_dir_images(image, prefs);
				let frame_dir_images_len = frame_dir_images.len();
				for dir_iteration in 0..frame_dir_images_len {
					let dir_image = frame_dir_images.remove(0); //we pop the first element each time
					images.insert(
						dir_iteration + (frame_iteration * frame_dir_images_len),
						dir_image,
					); //we need to re-arrange the order, so that it's one direction each time
				}
				frame_iteration += 1;
			}
			(frames_len, images, Some(delay))
		}
	};

	Ok(icon::IconState {
		name: file_name,
		dirs: 4,
		frames,
		images,
		delay,
		..Default::default()
	})
}

fn extract_four_dir_images(
	input_image: image::DynamicImage,
	prefs: &config::PrefHolder,
) -> Vec<image::DynamicImage> {
	let north_img = input_image.crop_imm(
		prefs.north_start_x,
		prefs.north_start_y,
		prefs.x_step,
		prefs.y_step,
	);
	let east_img = input_image.crop_imm(
		prefs.east_start_x,
		prefs.east_start_y,
		prefs.x_step,
		prefs.y_step,
	);
	let south_img = input_image.crop_imm(
		prefs.south_start_x,
		prefs.south_start_y,
		prefs.x_step,
		prefs.y_step,
	);
	let west_img = input_image.crop_imm(
		prefs.west_start_x,
		prefs.west_start_y,
		prefs.x_step,
		prefs.y_step,
	);
	vec![south_img, north_img, east_img, west_img]
}

///Takes everything that comes after the last slash (or backslash) in the string, discarding the rest.
fn trim_path_before_last_slash(mut text: String) -> String {
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
