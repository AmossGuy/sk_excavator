use egui::{Button, Context, IntoAtoms, MenuBar, TextWrapMode, Ui};
use super::app::ExcavatorContext;

use crate::misc::about::AboutWindow;
use crate::core::settings::SettingsWindow;

pub fn show_menu_bar_panel(ui: &mut Ui, excavator: &ExcavatorContext) {
	egui::Panel::top("menu bar").show(ui, |ui| {
		MenuBar::new().ui(ui, |ui| {
			file_menu_button(ui, excavator);
			edit_menu_button(ui, excavator);
			settings_menu_button(ui, excavator);
			help_menu_button(ui, excavator);
		});
	});
}

fn file_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("File", |ui| {
		menu_action(ui, excavator, "Open...", AppAction::OpenFile);
		ui.menu_button("Recent files", |ui| {
			recent_file_list(ui, excavator);
		});
		ui.separator();
		menu_action(ui, excavator, "Save", ViewAction::Save);
		menu_action(ui, excavator, "Save as...", ViewAction::SaveAs);
		ui.separator();
		menu_action(ui, excavator, "Quit", AppAction::Quit);
	});
}

fn edit_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("Edit", |ui| {
		menu_action(ui, excavator, "Undo", ViewAction::Undo);
		menu_action(ui, excavator, "Redo", ViewAction::Redo);
	});
}

fn settings_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("Settings", |ui| {
		menu_action(ui, excavator, "Configure Excavator...", AppAction::SettingsExcavator);
		menu_action(ui, excavator, "Configure egui...", AppAction::SettingsEgui);
	});
}

fn help_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("Help", |ui| {
		menu_action(ui, excavator, "About Excavator...", AppAction::About);
	});
}

fn menu_action<'a, A: MenuAction>(
	ui: &mut Ui, excavator: &ExcavatorContext,
	atoms: impl IntoAtoms<'a>, action: A,
) {
	let button = Button::new(atoms);
	/*
	if let Some(shortcut) = action.default_shortcut() {
		button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
	}
	*/
	
	let enabled = action.should_be_enabled(ui.ctx(), excavator);
	if ui.add_enabled(enabled, button).clicked() {
		action.execute(ui.ctx(), excavator);
	}
}

fn text_wrap_hack(ui: &mut Ui) {
	// egui's popup sizing stinks
	// this workaround prevents text wrapping in weird ways
	ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
}

fn recent_file_list(ui: &mut Ui, excavator: &ExcavatorContext) {
	text_wrap_hack(ui);
	
	let list = excavator.settings(|s| s.recent_files.iter().cloned().collect::<Vec<_>>());
	
	if list.is_empty() {
		ui.label("No recent files");
	} else {
		for item in list.into_iter().rev() {
			let file_name_string = item.file_name().unwrap_or_default().to_string_lossy();
			let response = ui.button(file_name_string);
			
			let response = response.on_hover_ui(|ui| {
				let full_path_string = item.to_string_lossy();
				ui.label(full_path_string);
			});
			
			if response.clicked() {
				excavator.open_file(item);
			}
		}
		
		ui.separator();
		menu_action(ui, excavator, "Clear recent files", AppAction::ClearRecentFiles);
	}
}

trait MenuAction {
	fn execute(&self, ctx: &Context, excavator: &ExcavatorContext);
	fn should_be_enabled(&self, ctx: &Context, excavator: &ExcavatorContext) -> bool;
}

#[derive(Copy, Clone, Debug)]
enum AppAction {
	OpenFile,
	ClearRecentFiles,
	Quit,
	
	SettingsExcavator,
	SettingsEgui,
	
	About,
}

impl MenuAction for AppAction {
	fn execute(&self, ctx: &Context, excavator: &ExcavatorContext) {
		match self {
			Self::OpenFile => excavator.open_file_dialog(),
			Self::ClearRecentFiles => excavator.settings_mut(|s| s.clear_recent_files()),
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			
			Self::SettingsExcavator => excavator.add_window(SettingsWindow::excavator_tab()),
			Self::SettingsEgui => excavator.add_window(SettingsWindow::egui_tab()),
			
			Self::About => excavator.add_window(AboutWindow::new()),
		}
	}
	
	fn should_be_enabled(&self, _ctx: &Context, _excavator: &ExcavatorContext) -> bool {
		true
	}
}

#[derive(Copy, Clone, Debug)]
pub enum ViewAction {
	Save,
	SaveAs,
	
	Undo,
	Redo,
}

impl MenuAction for ViewAction {
	fn execute(&self, _ctx: &Context, excavator: &ExcavatorContext) {
		if let Some(view) = excavator.get_file_view() {
			let mut view_lock = view.write();
			view_lock.menubar_execute(*self);
		}
	}
	
	fn should_be_enabled(&self, _ctx: &Context, excavator: &ExcavatorContext) -> bool {
		if let Some(view) = excavator.get_file_view() {
			let view_lock = view.read();
			view_lock.menubar_should_be_enabled(*self)
		} else {
			false
		}
	}
}
