use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use crate::file_view::common::tree::{TreeFileView, TreeFormatUi};
use excavator_backend::formats::pak::{def_live as pak, load_from_bytes};

use egui::Ui;
use std::sync::Arc;
use yoke::Yoke;

pub fn parse_pak(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let pak = load_from_bytes(&yoke_bytes)?;
	Ok(TreeFileView::new(pak))
}

impl TreeFormatUi for pak::Pak {
	fn item_ui(&self, ui: &mut Ui, item: pak::ItemId) {
		match item {
			pak::ItemId::Header(pak::HeaderId) => {
				egui::Grid::new("header fields").show(ui, |ui| {
					if let Some(edited_header) = edit_editable_data(ui, &self.header[0]) {
						// todo
						let _ = edited_header;
					}
				});
			},
		}
	}
}
