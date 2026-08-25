use egui::{Button, Context, IntoAtoms, MenuBar, TextWrapMode, Ui};
use super::app::ExcavatorContext;

use crate::misc::about::AboutWindow;
use crate::core::settings::SettingsWindow;

pub fn show_menu_bar_panel(ui: &mut Ui, excavator: &mut ExcavatorContext) {
	egui::Panel::top("menu bar").show(ui, |ui| {
		MenuBar::new().ui(ui, |ui| {
			file_menu_button(ui, excavator);
			settings_menu_button(ui, excavator);
			help_menu_button(ui, excavator);
		});
	});
}

fn file_menu_button(ui: &mut Ui, excavator: &mut ExcavatorContext) {
	ui.menu_button("File", |ui| {
		menu_action(ui, excavator, "Open...", MenuAction::OpenFile);
		ui.menu_button("Recent files", |ui| {
			recent_file_list(ui, excavator);
		});
		ui.separator();
		menu_action(ui, excavator, "Quit", MenuAction::Quit);
	});
}

fn settings_menu_button(ui: &mut Ui, excavator: &mut ExcavatorContext) {
	ui.menu_button("Settings", |ui| {
		menu_action(ui, excavator, "Configure Excavator...", MenuAction::SettingsExcavator);
		menu_action(ui, excavator, "Configure egui...", MenuAction::SettingsEgui);
	});
}

fn help_menu_button(ui: &mut Ui, excavator: &mut ExcavatorContext) {
	ui.menu_button("Help", |ui| {
		menu_action(ui, excavator, "About Excavator...", MenuAction::About);
	});
}

fn menu_action<'a>(
	ui: &mut Ui, excavator: &mut ExcavatorContext,
	atoms: impl IntoAtoms<'a>, action: MenuAction,
) {
	let button = Button::new(atoms);
	/*
	if let Some(shortcut) = action.default_shortcut() {
		button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
	}
	*/
	if ui.add(button).clicked() {
		action.execute(ui.ctx(), excavator);
	}
}

fn text_wrap_hack(ui: &mut Ui) {
	// egui's popup sizing stinks
	// this workaround prevents text wrapping in weird ways
	ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
}

fn recent_file_list(ui: &mut Ui, excavator: &mut ExcavatorContext) {
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
		menu_action(ui, excavator, "Clear recent files", MenuAction::ClearRecentFiles);
	}
}

#[derive(Copy, Clone, Debug)]
pub enum MenuAction {
	OpenFile,
	ClearRecentFiles,
	Quit,
	
	SettingsExcavator,
	SettingsEgui,
	
	About,
}

impl MenuAction {
	fn execute(&self, ctx: &Context, excavator: &mut ExcavatorContext) {
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
