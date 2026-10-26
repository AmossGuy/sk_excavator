use super::AnbFileView;
use crate::app::context::ExcavatorContext;

use egui::Ui;

pub struct EditPanelState {
	tab: EditPanelTab,
}

impl EditPanelState {
	pub fn new() -> Self {
		Self {
			tab: EditPanelTab::File,
		}
	}
	
	pub fn show_node_tab(&mut self) {
		self.tab = EditPanelTab::Node;
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum EditPanelTab {
	File,
	Node,
}

pub fn edit_panel_ui(anb_view: &mut AnbFileView, ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.horizontal(|ui| {
		ui.selectable_value(&mut anb_view.edit_panel.tab, EditPanelTab::File, "File");
		ui.selectable_value(&mut anb_view.edit_panel.tab, EditPanelTab::Node, "Node");
	});
	
	ui.separator();
	
	match anb_view.edit_panel.tab {
		EditPanelTab::File => { file_tab_ui(anb_view, ui, excavator); },
		EditPanelTab::Node => { node_tab_ui(anb_view, ui, excavator); },
	}
}

fn file_tab_ui(anb_view: &mut AnbFileView, ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.label("file tab placeholder");
}

fn node_tab_ui(anb_view: &mut AnbFileView, ui: &mut Ui, excavator: &ExcavatorContext) {
	ui.label("node tab placeholder");
}
