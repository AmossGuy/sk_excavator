use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::file_view::FileBytes;

pub fn hexedit_ui(bytes: &FileBytes, ui: &mut Ui) {
	let column_count = 0x10;
	
	let mut table = TableBuilder::new(ui);
	table = table.striped(true);
	
	for _ in 0..column_count {
		table = table.column(Column::remainder().clip(true));
	}
	
	table.header(20.0, |mut table_header| {
		for col_n in 0..column_count {
			table_header.col(|ui| {
				ui.strong(format!("{:X}", col_n));
			});
		}
	});
}
