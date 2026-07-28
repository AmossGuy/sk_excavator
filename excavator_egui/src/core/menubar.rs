use egui::{Key, KeyboardShortcut, Modifiers};
use super::app::ExcavatorContext;
use super::menu::{Menu, MenuAction, MenuItem, RootMenu};

pub fn show_menu_bar_panel(ui: &mut egui::Ui, env: &mut ExcavatorContext) {
	egui::Panel::top("menu bar").show_inside(ui, |ui| {
		egui::MenuBar::new().ui(ui, |ui| {
			MENU_BAR.ui(ui, env);
		});
	});
}

static MENU_BAR: RootMenu<MenuBarAction> = RootMenu::new(&[
	MenuItem::SubMenu(Menu::new("File", &[
		MenuItem::Action(MenuBarAction::SelectGameDir),
		MenuItem::Action(MenuBarAction::CloseGameDir),
		MenuItem::SubMenu(Menu::new("Recent files", &[
			MenuItem::Separator,
			MenuItem::Action(MenuBarAction::ClearRecentFiles),
		])),
		MenuItem::Separator,
		MenuItem::Action(MenuBarAction::Quit),
	])),
	/*
	MenuItem::SubMenu(Menu::new("Edit", &[
		MenuItem::Action(MenuBarAction::Undo),
		MenuItem::Action(MenuBarAction::Redo),
		MenuItem::Separator,
		MenuItem::Action(MenuBarAction::Settings),
	])),
	*/
	MenuItem::SubMenu(Menu::new("Help", &[
		MenuItem::Action(MenuBarAction::About),
	])),
]);

#[derive(Copy, Clone, Debug)]
pub enum MenuBarAction {
	SelectGameDir,
	CloseGameDir,
	ClearRecentFiles,
	Quit,
	
	/*
	Undo,
	Redo,
	Settings,
	*/
	
	About,
}

impl MenuAction for MenuBarAction {
	type Env = ExcavatorContext;
	
	fn static_name(&self) -> &'static str {
		match self {
			Self::SelectGameDir => "Select game directory...",
			Self::CloseGameDir => "Close game directory",
			Self::ClearRecentFiles => "Clear recent files",
			Self::Quit => "Quit",
			
			/*
			Self::Undo => "Undo",
			Self::Redo => "Redo",
			Self::Settings => "Settings...",
			*/
			
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
			Self::SelectGameDir => crate::misc::file_dialog::show_game_path_dialog(ctx, excavator),
			Self::CloseGameDir => excavator.settings_mut(|s| s.game_root_path = None),
			Self::ClearRecentFiles => {},
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			
			Self::About => excavator.add_window(crate::misc::about::AboutWindow::new()),
		}
	}
}
