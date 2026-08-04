use excavator_backend::io::file::FileSource;
use crate::core::app::ExcavatorContext;
use crate::core::windows::Window;

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

pub struct SettingsWindow {
	tab: SettingsTab,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SettingsTab {
	Excavator,
	Egui,
}

impl SettingsWindow {
	pub fn excavator_tab() -> Self {
		Self { tab: SettingsTab::Excavator }
	}
	
	pub fn egui_tab() -> Self {
		Self { tab: SettingsTab::Egui }
	}
}

impl Window for SettingsWindow {
	fn ui(&mut self, ui: &mut egui::Ui, excavator: &ExcavatorContext) {
		egui::Panel::top("setting tabs").show_inside(ui, |ui| {
			ui.horizontal(|ui| {
				ui.selectable_value(&mut self.tab, SettingsTab::Excavator, "Excavator");
				ui.selectable_value(&mut self.tab, SettingsTab::Egui, "egui");
			});
		});
		
		egui::CentralPanel::default().show_inside(ui, |ui| {
			match self.tab {
				SettingsTab::Excavator => {
					egui::ScrollArea::vertical().show(ui, |ui| {
						egui::Grid::new("settings grid").show(ui, |ui| {
							ui.label("Maximum number of recent files");
							excavator.settings_mut(|s| {
								ui.add(egui::DragValue::new(&mut s.max_recent_files));
							});
							ui.end_row();
						});
					});
				},
				SettingsTab::Egui => {
					egui::ScrollArea::vertical().show(ui, |ui| {
						ui.ctx().clone().settings_ui(ui);
					});
				},
			}
		});
	}
	
	fn initial_size(&self) -> egui::Vec2 {
		egui::Vec2::new(500.0, 550.0)
	}
}
