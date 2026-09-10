mod shortcuts;
pub use shortcuts::ShortcutStorage;

use egui::{Button, Context, IntoAtoms, MenuBar, TextWrapMode, Ui};
use crate::app::{about::AboutWindow, context::ExcavatorContext, settings::SettingsWindow};

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

pub fn test_menu_bar_shortcuts(ctx: &Context, excavator: &ExcavatorContext) {
	if let Some(action) = excavator.shortcuts(|s| s.test_shortcuts(ctx)) {
		action.execute(ctx, excavator);
	}
}

fn file_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("File", |ui| {
		menu_action(ui, excavator, "Open...", MenuAction::OpenFile);
		ui.menu_button("Open recent", |ui| {
			recent_file_list(ui, excavator);
		});
		ui.separator();
		menu_action(ui, excavator, "Save", MenuAction::Save);
		menu_action(ui, excavator, "Save as...", MenuAction::SaveAs);
		ui.separator();
		menu_action(ui, excavator, "Quit", MenuAction::Quit);
	});
}

fn edit_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("Edit", |ui| {
		menu_action(ui, excavator, "Undo", MenuAction::Undo);
		menu_action(ui, excavator, "Redo", MenuAction::Redo);
	});
}

fn settings_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("Settings", |ui| {
		menu_action(ui, excavator, "Configure Excavator...", MenuAction::SettingsExcavator);
		menu_action(ui, excavator, "Configure egui...", MenuAction::SettingsEgui);
	});
}

fn help_menu_button(ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.menu_button("Help", |ui| {
		menu_action(ui, excavator, "About Excavator...", MenuAction::About);
	});
}

fn menu_action<'a>(
	ui: &mut Ui, excavator: &ExcavatorContext,
	atoms: impl IntoAtoms<'a>, action: MenuAction,
) {
	let mut button = Button::new(atoms);
	if let Some(shortcut) = excavator.shortcuts(|s| s.get_action_shortcut(action)) {
		button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
	}
	
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
					excavator.open_file(item);
				}
			}
			
			ui.separator();
			menu_action(ui, excavator, "Clear recent files", MenuAction::ClearRecentFiles);
		});
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
	fn execute(&self, ctx: &Context, excavator: &ExcavatorContext) {
		match self {
			Self::OpenFile => excavator.open_file_dialog(),
			Self::ClearRecentFiles => excavator.settings_mut(|s| s.clear_recent_files()),
			Self::Save => {}, // todo
			Self::SaveAs => {}, // todo
			Self::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
			
			Self::Undo => excavator.file_view_mut(|v| v.execute_undo()).unwrap_or(()),
			Self::Redo => excavator.file_view_mut(|v| v.execute_redo()).unwrap_or(()),
			
			Self::SettingsExcavator => excavator.add_window(SettingsWindow::excavator_tab()),
			Self::SettingsEgui => excavator.add_window(SettingsWindow::egui_tab()),
			
			Self::About => excavator.add_window(AboutWindow::new()),
		}
	}
	
	fn should_be_enabled(&self, _ctx: &Context, excavator: &ExcavatorContext) -> bool {
		match self {
			Self::Save => false, // todo
			Self::SaveAs => false, // todo
			Self::Undo => excavator.file_view(|v| v.can_undo()).unwrap_or(false),
			Self::Redo => excavator.file_view(|v| v.can_redo()).unwrap_or(false),
			_ => true,
		}
	}
}
