use egui::{Key, KeyboardShortcut, Modifiers};
use super::app::ExcavatorApp;
use super::menu::{Menu, MenuAction, MenuItem, RootMenu};

pub fn show_menu_bar_panel(ui: &mut egui::Ui, app: &mut ExcavatorApp, frame: &mut eframe::Frame) {
	egui::Panel::top("menu bar").show_inside(ui, |ui| {
		egui::MenuBar::new().ui(ui, |ui| {
			MENU_BAR.ui(ui, &mut |action, ctx| action.apply(app, ctx, frame));
		});
	});
}

static MENU_BAR: RootMenu<MenuBarAction> = RootMenu::new(&[
	MenuItem::SubMenu(Menu::new("File", &[
		MenuItem::Action(MenuBarAction::SelectGameDir),
		MenuItem::Action(MenuBarAction::CloseGameDir),
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
	
	Quit,
	
	/*
	Undo,
	Redo,
	Settings,
	*/
	
	About,
}

impl MenuAction for MenuBarAction {
	fn static_name(&self) -> &'static str {
		match self {
			Self::SelectGameDir => "Select game directory...",
			Self::CloseGameDir => "Close game directory",
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
}

impl MenuBarAction {
	pub fn apply(self, app: &mut ExcavatorApp, ctx: &egui::Context, frame: &mut eframe::Frame) {
		match self {
			Self::SelectGameDir => crate::misc::file_dialog::show_game_path_dialog(app, ctx, frame),
			Self::CloseGameDir => app.set_game_root_path(None),
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			Self::About => app.windows.add(crate::misc::about::AboutWindow::new()),
		}
	}
}
