use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::pak::{self, Pak, load_from_bytes};

use egui::{Id, Label, ScrollArea, Ui};
use egui_ltreeview::{NodeConfig, TreeView, TreeViewState};
use std::sync::Arc;
use yoke::Yoke;

pub fn parse_pak(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let pak = load_from_bytes(&yoke_bytes)?;
	Ok(PakFileView::new(pak))
}

struct PakFileView {
	pak: Pak,
	tree_state: TreeViewState<pak::FileId>,
}

impl FileView for PakFileView {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext) {
		egui::Panel::right("property editor").show(ui, |ui| {
			self.property_view(ui);
			ui.take_available_space();
		});
		
		egui::CentralPanel::default().show(ui, |ui| {
			self.tree_view(ui);
		});
	}
}

impl PakFileView {
	fn new(pak: Pak) -> Self {
		Self { pak, tree_state: TreeViewState::default() }
	}
	
	fn tree_view(&mut self, ui: &mut Ui) {
		ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			TreeView::new(Id::new("tree view")).show_state(ui, &mut self.tree_state, |builder| {
				for &file_id in &self.pak.get_header().files {
					let config = PakNodeConfig::from_pak_and_id(&self.pak, file_id).expect("file should exist");
					builder.node(config);
				}
			});
		});
	}
	
	fn property_view(&mut self, ui: &mut Ui) {
		match self.tree_state.selected().as_slice() {
			// I need to change this in some way, because the tree view does not provide a convenient way to deselect everything. I'm thinking tab buttons.
			&[] => {
				egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
					edit_editable_data(ui, self.pak.get_header());
				});
			},
			&[file_id] => {
				let file = self.pak.get_file(file_id).expect("file should exist");
				egui::Grid::new("property grid").num_columns(2).show(ui, |ui| {
					edit_editable_data(ui, file);
				});
			},
			_ => {
				ui.label("multiple selected");
			},
		}
	}
}

struct PakNodeConfig<'a> {
	id: pak::FileId,
	value: &'a pak::File,
}

impl<'a> PakNodeConfig<'a> {
	fn from_pak_and_id(pak: &'a Pak, id: pak::FileId) -> Option<Self> {
		let value = pak.get_file(id)?;
		Some(Self { id, value })
	}
}

impl<'a> NodeConfig<pak::FileId> for PakNodeConfig<'a> {
	fn id(&self) -> &pak::FileId {
		&self.id
	}
	
	fn is_dir(&self) -> bool {
		false
	}
	
	fn label(&mut self, ui: &mut Ui) {
		let text = String::from_utf8_lossy(self.value.filename.get());
		ui.add(Label::new(text).selectable(false));
	}
}
