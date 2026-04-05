use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

use excavator_backend::formats::anb::{parse_anb, ParsedAnb, ParsedAnbNode};
use excavator_backend::parse::ParseResult;

pub struct AnbFileView {
	parsed: ParseResult<ParsedAnb>,
}

impl ItemView for AnbFileView {
	fn new(bytes: FileBytes, _ctx: &egui::Context) -> Self where Self: Sized {
		let mut cursor = std::io::Cursor::new(bytes.as_slice());
		// blocks the main thread for now, until i figure out an ergonomic system for all the threading this app ought to do
		let parsed = parse_anb(&mut cursor);
		Self { parsed }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		match &self.parsed {
			Ok(parsed) => {
				egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
					node_ui(ui, "root", parsed.root());
				});
			},
			Err(e) => { ui.label(format!("failed to load anb: {}", e)); },
		};
		None
	}
}

fn node_ui(ui: &mut egui::Ui, index: impl std::hash::Hash + std::fmt::Display, node: &ParseResult<ParsedAnbNode>) {
	match node {
		Ok(node) => {
			egui::CollapsingHeader::new(format!("{} (kind: {})", index, node.kind()))
				.id_salt(index)
				.default_open(true)
				.show(ui, |ui| {
					for (i, child) in node.children().enumerate() {
						node_ui(ui, i, child);
					}
				});
		},
		Err(e) => { ui.label(format!("error reading node: {}", e)); },
	}
}
