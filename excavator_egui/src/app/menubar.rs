mod shortcuts;
pub use shortcuts::ShortcutStorage;

use egui::{Button, Context, IntoAtoms, MenuBar, TextWrapMode, Ui};
use crate::file_view::{FileView, FileViewAction};
use crate::app::{about::AboutWindow, context::ExcavatorContext, settings::SettingsWindow};

pub struct MenuEnv<'a> {
	excavator: &'a ExcavatorContext,
	file_view: &'a mut dyn FileView,
}

impl<'a> MenuEnv<'a> {
	pub fn new(
		excavator: &'a ExcavatorContext,
		file_view: &'a mut dyn FileView,
	) -> Self {
		Self { excavator, file_view }
	}
}

pub fn show_menu_bar_panel(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	egui::Panel::top("menu bar").show(ui, |ui| {
		MenuBar::new().ui(ui, |ui| {
			file_menu_button(ui, env);
			edit_menu_button(ui, env);
			settings_menu_button(ui, env);
			help_menu_button(ui, env);
		});
	});
}

pub fn test_menu_bar_shortcuts(ctx: &Context, env: &mut MenuEnv<'_>) {
	if let Some(action) = env.excavator.shortcuts(|s| s.test_shortcuts(ctx)) {
		action.execute(ctx, env);
	}
}

fn file_menu_button(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	ui.menu_button("File", |ui| {
		menu_action(ui, env, "Open...", MenuAction::OpenFile);
		ui.menu_button("Open recent", |ui| {
			recent_file_list(ui, env);
		});
		ui.separator();
		menu_action(ui, env, "Save", MenuAction::Save);
		menu_action(ui, env, "Save as...", MenuAction::SaveAs);
		ui.separator();
		menu_action(ui, env, "Quit", MenuAction::Quit);
	});
}

fn edit_menu_button(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	ui.menu_button("Edit", |ui| {
		menu_action(ui, env, "Undo", MenuAction::Undo);
		menu_action(ui, env, "Redo", MenuAction::Redo);
		ui.menu_button("Undo history", |ui| {
			undo_history_list(ui, env);
		});
	});
}

fn settings_menu_button(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	ui.menu_button("Settings", |ui| {
		menu_action(ui, env, "Configure Excavator...", MenuAction::SettingsExcavator);
		menu_action(ui, env, "Configure egui...", MenuAction::SettingsEgui);
	});
}

fn help_menu_button(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	ui.menu_button("Help", |ui| {
		menu_action(ui, env, "About Excavator...", MenuAction::About);
	});
}

fn menu_action<'a>(
	ui: &mut Ui, env: &mut MenuEnv<'_>,
	atoms: impl IntoAtoms<'a>, action: MenuAction,
) {
	let mut button = Button::new(atoms);
	if let Some(shortcut) = env.excavator.shortcuts(|s| s.get_action_shortcut(action)) {
		button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
	}
	
	let enabled = action.should_be_enabled(ui.ctx(), env);
	if ui.add_enabled(enabled, button).clicked() {
		action.execute(ui.ctx(), env);
	}
}

fn text_wrap_hack(ui: &mut Ui) {
	// egui's popup sizing stinks
	// this workaround prevents text wrapping in weird ways
	ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
}

fn recent_file_list(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	text_wrap_hack(ui);
	
	let list = env.excavator.settings(|s| s.recent_files.iter().cloned().collect::<Vec<_>>());
	
	if list.is_empty() {
		ui.add(egui::Label::new("No recent files").selectable(false));
	} else {
		egui::ScrollArea::vertical().show(ui, |ui| {
			for item in list.into_iter().rev() {
				let file_name_string = item.file_name().unwrap_or_default().to_string_lossy();
				let response = ui.button(file_name_string);
				
				let response = response.on_hover_ui(|ui| {
					let full_path_string = item.to_string_lossy();
					ui.label(full_path_string);
				});
				
				if response.clicked() {
					env.excavator.open_file(item);
				}
			}
			
			ui.separator();
			menu_action(ui, env, "Clear recent files", MenuAction::ClearRecentFiles);
		});
	}
}

fn undo_history_list(ui: &mut Ui, env: &mut MenuEnv<'_>) {
	text_wrap_hack(ui);
	
	if let Some(undo_history) = env.file_view.undo_history() && !undo_history.is_empty() {
		egui::ScrollArea::vertical().show(ui, |ui| {
			for (new_index, text) in undo_history.iter().enumerate().rev() {
				let index = env.file_view.undo_history_index();
				if ui.add(egui::Button::selectable(index == Some(new_index), text.as_ref())).clicked() {
					env.file_view.undo_go_to_index(new_index);
				}
			}
		});
	} else {
		ui.add(egui::Label::new("No undo history").selectable(false));
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum MenuAction {
	// file
	OpenFile,
	ClearRecentFiles,
	Save,
	SaveAs,
	Quit,
	
	// edit
	Undo,
	Redo,
	
	// settings
	SettingsExcavator,
	SettingsEgui,
	
	// help
	About,
}

impl MenuAction {
	fn execute(&self, ctx: &Context, env: &mut MenuEnv<'_>) {
		match self {
			Self::OpenFile => env.excavator.open_file_dialog(),
			Self::ClearRecentFiles => env.excavator.settings_mut(|s| s.clear_recent_files()),
			Self::Save => env.file_view.action_execute(FileViewAction::Save, env.excavator),
			Self::SaveAs => env.file_view.action_execute(FileViewAction::SaveAs, env.excavator),
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			
			Self::Undo => env.file_view.action_execute(FileViewAction::Undo, env.excavator),
			Self::Redo => env.file_view.action_execute(FileViewAction::Redo, env.excavator),
			
			Self::SettingsExcavator => env.excavator.add_window(SettingsWindow::excavator_tab()),
			Self::SettingsEgui => env.excavator.add_window(SettingsWindow::egui_tab()),
			
			Self::About => env.excavator.add_window(AboutWindow::new()),
		}
	}
	
	fn should_be_enabled(&self, _ctx: &Context, env: &mut MenuEnv<'_>) -> bool {
		match self {
			Self::Save => env.file_view.action_should_be_enabled(FileViewAction::Save),
			Self::SaveAs => env.file_view.action_should_be_enabled(FileViewAction::SaveAs),
			
			Self::Undo => env.file_view.action_should_be_enabled(FileViewAction::Undo),
			Self::Redo => env.file_view.action_should_be_enabled(FileViewAction::Redo),
			
			_ => true,
		}
	}
}
