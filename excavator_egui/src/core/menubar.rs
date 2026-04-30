use egui::{Key, KeyboardShortcut, Modifiers};
use super::app::ExcavatorApp;
use super::menu::{Menu, MenuAction, MenuItem, RootMenu};
use super::message::Message;

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

#[derive(Copy, Clone, Debug)]
pub enum MenuBarAction {
	SelectGamePath,
	Quit,
	
	Undo,
	Redo,
	Settings,
	
	About,
}

impl MenuAction for MenuBarAction {
	fn static_name(&self) -> &'static str {
		match self {
			Self::SelectGamePath => "Select game path...",
			Self::Quit => "Quit",
			
			Self::Undo => "Undo",
			Self::Redo => "Redo",
			Self::Settings => "Settings...",
			
			Self::About => "About Shovel Knight Excavator",
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
	
	fn into_message(&self) -> Message {
		Message::MenuBarAction(*self)
	}
}

impl MenuBarAction {
	pub fn apply(self, ctx: &egui::Context, app: &mut ExcavatorApp) {
		println!("menubar: {:?}", self);
		match self {
			Self::SelectGamePath => crate::misc::file_dialog::show_game_path_dialog(ctx),
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			Self::About => app.windows.add(crate::misc::about::AboutWindow::new()),
			_ => {}, // TODO: implement all menu items
		}
	}
}
