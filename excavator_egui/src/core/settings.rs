use excavator_backend::io::file::FileSource;

use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ExcavatorSettings {
	pub game_root_path: Option<PathBuf>,
	pub recent_files: VecDeque<FileSource>,
	pub max_recent_files: u8,
}

impl Default for ExcavatorSettings {
	fn default() -> Self {
		Self {
			game_root_path: None,
			recent_files: VecDeque::new(),
			max_recent_files: 10,
		}
	}
}

impl ExcavatorSettings {
	pub fn load(storage: &dyn eframe::Storage) -> Self {
		eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
	}
	
	pub fn save(&self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
}
