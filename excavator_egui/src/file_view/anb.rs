use crate::file_view::FileView;
use crate::file_view::common::editable::edit_editable_data;
use crate::file_view::common::tree::{TreeFileView, TreeFormatUi};
use excavator_backend::formats::anb::{def_live as anb, load_from_bytes};
// use excavator_backend::formats::wflz;

use egui::Ui;
use std::sync::Arc;
use yoke::Yoke;

pub fn parse_anb(file_contents: Vec<u8>) -> anyhow::Result<impl FileView> {
	let yoke_bytes = Yoke::attach_to_cart(Arc::new(file_contents), |vec| &vec[..]);
	let anb = load_from_bytes(&yoke_bytes)?;
	Ok(TreeFileView::new(anb))
}

impl TreeFormatUi for anb::Anb {
	fn item_ui(&self, ui: &mut Ui, item: anb::AnyItemRef) {
		match item {
			anb::AnyItemRef::Header(header) => {
				egui::Grid::new("header fields").show(ui, |ui| {
					if let Some(edited_header) = edit_editable_data(ui, &header.data) {
						// todo
						let _ = edited_header;
					}
				});
			},
		}
	}
}
