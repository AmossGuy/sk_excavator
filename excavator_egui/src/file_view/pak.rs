use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use excavator_backend::formats::pak::{def_live as pak, def_live::Pak, load_from_bytes};

use egui::{Id, Label, ScrollArea, Ui};
use egui_ltreeview::{NodeConfig, TreeView};
use std::sync::Arc;
use yoke::Yoke;

pub fn parse_pak(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let pak = load_from_bytes(&yoke_bytes)?;
	Ok(PakFileView::new(pak))
}

struct PakFileView {
	pak: Pak,
}

impl FileView for PakFileView {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext) {
		self.tree_view(ui);
	}
}

impl PakFileView {
	fn new(pak: Pak) -> Self {
		Self { pak }
	}
	
	fn tree_view(&mut self, ui: &mut Ui) {
		ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			TreeView::new(Id::new("tree view")).show(ui, |builder| {
				for &file_id in &self.pak.get_header().files {
					let config = PakNodeConfig::from_pak_and_id(&self.pak, file_id).expect("file should exist");
					builder.node(config);
				}
			});
		});
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
		ui.add(Label::new("wip").selectable(false));
	}
}
