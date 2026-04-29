use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct ExcavatorSettings {
	pub game_root_path: Option<PathBuf>,
}

impl ExcavatorSettings {
	pub fn load(storage: &dyn eframe::Storage) -> Self {
		eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
	}
	
	pub fn save(&self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}
}
