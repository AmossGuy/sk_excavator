use egui::{Key, KeyboardShortcut, Modifiers};
use super::menu::{Menu, MenuAction, MenuItem, RootMenu};

pub fn show_menu_bar_panel(ui: &mut egui::Ui) {
	egui::Panel::top("menu bar").show_inside(ui, |ui| {
		egui::MenuBar::new().ui(ui, |ui| {
			MENU_BAR.ui(ui);
		});
	});
}

static MENU_BAR: RootMenu<MenuBarAction> = RootMenu::new(&[
	MenuItem::SubMenu(Menu::new("File", &[
		MenuItem::Action(MenuBarAction::SelectGamePath),
		MenuItem::Separator,
		MenuItem::Action(MenuBarAction::Quit),
	])),
	MenuItem::SubMenu(Menu::new("Edit", &[
		MenuItem::Action(MenuBarAction::Undo),
		MenuItem::Action(MenuBarAction::Redo),
		MenuItem::Separator,
		MenuItem::Action(MenuBarAction::Settings),
	])),
	MenuItem::SubMenu(Menu::new("Help", &[
		MenuItem::Action(MenuBarAction::About),
	])),
]);

#[derive(Copy, Clone)]
pub enum MenuBarAction {
	SelectGamePath,
	Quit,
	
	Undo,
	Redo,
	Settings,
	
	About,
}

impl MenuAction for MenuBarAction {
	fn name(&self, _ctx: &egui::Context) -> String {
		match self {
			Self::SelectGamePath => "Select game path...".to_string(),
			Self::Quit => "Quit".to_string(),
			
			Self::Undo => "Undo".to_string(),
			Self::Redo => "Redo".to_string(),
			Self::Settings => "Settings...".to_string(),
			
			Self::About => "About Shovel Knight Excavator".to_string(),
		}
	}
	
	fn default_shortcut(&self) -> Option<KeyboardShortcut> {
		type KS = KeyboardShortcut;
		type Mod = Modifiers;
		
		match self {
			Self::Quit => Some(KS::new(Mod::COMMAND, Key::Q)),
			Self::Undo => Some(KS::new(Mod::COMMAND, Key::Z)),
			Self::Redo => Some(KS::new(Mod::COMMAND | Mod::SHIFT, Key::Z)),
			_ => None,
		}
	}
}
