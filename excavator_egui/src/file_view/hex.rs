use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::file_view::FileBytes;

pub fn hexedit_ui(bytes: &FileBytes, ui: &mut Ui) {
	let column_count = 0x10;
	
	let text_height = egui::TextStyle::Body
		.resolve(ui.style())
		.size.max(ui.spacing().interact_size.y);
	
	let mut table = TableBuilder::new(ui);
	table = table.striped(true);
	
	for _ in 0..column_count {
		table = table.column(Column::remainder().clip(true));
	}
	
	let slice = bytes.as_slice();
	
	table.header(20.0, |mut table_header| {
		for col_n in 0..column_count {
			table_header.col(|ui| {
				ui.strong(format!("{:X}", col_n));
			});
		}
	}).body(|body| {
		body.rows(text_height, slice.len() / column_count, |mut row| {
			let start = row.index() * column_count;
			let subslice = slice.get(start..(start + column_count)).unwrap_or_default();
			for byte in subslice {
				row.col(|ui| { ui.label(format!("{:02X}", byte)); });
			}
		});
	});
}
