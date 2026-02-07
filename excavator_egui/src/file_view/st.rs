use egui::Ui;
use egui_extras::{Column, TableBuilder};
use std::io::Cursor;

use crate::ExcavatorMessage;
use crate::file_view::FileBytes;
use excavator_formats::st::{read_st_header, read_st_cell};

pub struct StFileView {
	bytes: FileBytes,
	is_stl: bool,
}

impl super::ItemView for StFileView {
	fn new(bytes: FileBytes, _ctx: &egui::Context) -> Self where Self: Sized {
		let is_stl = bytes.source_item().extension() == Some(b"stl");
		Self { bytes, is_stl }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		egui::ScrollArea::horizontal().show(ui, |ui| {
			self.table_ui(ui);
		});
		None
	}
}

impl StFileView {
	
	fn table_ui(&mut self, ui: &mut Ui) {
		let data = self.bytes.as_slice();
		let mut cursor = Cursor::new(data);
		let st_header = read_st_header(&mut cursor, self.is_stl).unwrap();
		
		let text_height = egui::TextStyle::Body
			.resolve(ui.style())
			.size.max(ui.spacing().interact_size.y);
		
		let mut table = TableBuilder::new(ui);
		table = table.striped(true);
		
		for _ in 0..st_header.field_count {
			table = table.column(Column::remainder().clip(true));
		}
		
		table.header(20.0, |mut table_header| {
			for col_n in 0..st_header.field_count as usize {
				table_header.col(|ui| {
					let cell_n = col_n;
					let string = read_st_cell(&mut cursor, &st_header, cell_n).unwrap().to_string();
					ui.strong(string);
				});
			}
		}).body(|body| {
			body.rows(text_height, st_header.entry_count as usize - 1, |mut row| {
				let row_n = row.index() + 1;
				for col_n in 0..st_header.field_count as usize {
					row.col(|ui| {
						let cell_n = row_n * st_header.field_count as usize + col_n;
						let string = read_st_cell(&mut cursor, &st_header, cell_n).unwrap().to_string();
						ui.label(string);
					});
				}
			});
		});
	}
}
