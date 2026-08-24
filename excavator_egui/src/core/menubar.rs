use egui::{Key, KeyboardShortcut, Modifiers};
use super::app::ExcavatorContext;
use super::menu::{MenuAction, MenuItem, RootMenu};

use crate::misc::about::AboutWindow;
use crate::core::settings::SettingsWindow;

pub fn show_menu_bar_panel(ui: &mut egui::Ui, env: &mut ExcavatorContext) {
	egui::Panel::top("menu bar").show(ui, |ui| {
		egui::MenuBar::new().ui(ui, |ui| {
			MENU_BAR.ui(ui, env);
		});
	});
}

static MENU_BAR: RootMenu<MenuBarAction> = RootMenu::new(&[
	MenuItem::SubMenu("File", &[
		MenuItem::Action(MenuBarAction::OpenFile),
		MenuItem::SubMenu("Recent files", &[
			MenuItem::CustomUi(recent_file_list),
			MenuItem::CustomCondition(recent_files_not_empty, &[
				MenuItem::Separator,
				MenuItem::Action(MenuBarAction::ClearRecentFiles),
			]),
		]),
		MenuItem::Separator,
		MenuItem::Action(MenuBarAction::Quit),
	]),
	MenuItem::SubMenu("Settings", &[
		MenuItem::Action(MenuBarAction::SettingsExcavator),
		MenuItem::Action(MenuBarAction::SettingsEgui),
	]),
	MenuItem::SubMenu("Help", &[
		MenuItem::Action(MenuBarAction::About),
	]),
]);

#[derive(Copy, Clone, Debug)]
pub enum MenuBarAction {
	OpenFile,
	ClearRecentFiles,
	Quit,
	
	SettingsExcavator,
	SettingsEgui,
	
	About,
}

impl MenuAction for MenuBarAction {
	type Env = ExcavatorContext;
	
	fn static_name(&self) -> &'static str {
		match self {
			Self::OpenFile => "Open...",
			Self::ClearRecentFiles => "Clear recent files",
			Self::Quit => "Quit",
			
			Self::SettingsExcavator => "Configure Excavator...",
			Self::SettingsEgui => "Configure egui...",
			
			Self::About => "About Shovel Knight Excavator",
		}
	}
	
	fn default_shortcut(&self) -> Option<KeyboardShortcut> {
		type KS = KeyboardShortcut;
		type Mod = Modifiers;
		
		match self {
			Self::Quit => Some(KS::new(Mod::COMMAND, Key::Q)),
			/*
			Self::Undo => Some(KS::new(Mod::COMMAND, Key::Z)),
			Self::Redo => Some(KS::new(Mod::COMMAND | Mod::SHIFT, Key::Z)),
			*/
			_ => None,
		}
	}
	
	fn execute(&self, ctx: &egui::Context, excavator: &mut ExcavatorContext) {
		match self {
			Self::OpenFile => excavator.open_file_dialog(),
			Self::ClearRecentFiles => excavator.settings_mut(|s| s.clear_recent_files()),
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			
			Self::SettingsExcavator => excavator.add_window(SettingsWindow::excavator_tab()),
			Self::SettingsEgui => excavator.add_window(SettingsWindow::egui_tab()),
			
			Self::About => excavator.add_window(AboutWindow::new()),
		}
	}
}

fn recent_file_list(ui: &mut egui::Ui, excavator: &mut ExcavatorContext) {
	let list = excavator.settings(|s| s.recent_files.clone());
	if list.is_empty() {
		ui.add_enabled_ui(false, |ui| {
			ui.label("No recent files");
		});
	} else {
		for item in list.into_iter().rev() {
			let file_name_string = item.file_name().unwrap_or_default().to_string_lossy();
			if ui.button(file_name_string).clicked() {
				excavator.open_file(item);
			}
		}
	}
}

fn recent_files_not_empty(_ctx: &egui::Context, excavator: &mut ExcavatorContext) -> bool {
	excavator.settings(|s| !s.recent_files.is_empty())
}
