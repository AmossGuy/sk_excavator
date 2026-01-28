use egui::Ui;
use egui_extras::{Column, TableBuilder};
use std::io::Cursor;

use excavator_formats::st::{read_st_header, read_st_cell};

#[derive(Default)]
pub struct StFileView;

impl StFileView {
	pub fn view_ui(&mut self, ui: &mut Ui, data: &[u8], is_stl: bool) {
		egui::ScrollArea::horizontal().show(ui, |ui| {
			self.table_ui(ui, data, is_stl);
		});
	}
	
	fn table_ui(&mut self, ui: &mut Ui, data: &[u8], is_stl: bool) {
		let mut cursor = Cursor::new(data);
		let st_header = read_st_header(&mut cursor, is_stl).unwrap();
		
		let text_height = egui::TextStyle::Body
			.resolve(ui.style())
			.size.max(ui.spacing().interact_size.y);
		
		let mut table = TableBuilder::new(ui);
		
		for _ in 0..st_header.field_count {
			table = table.column(Column::remainder());
		}
		
		table.header(20.0, |mut table_header| {
			for col_n in 0..st_header.field_count {
				table_header.col(|ui| {
					ui.strong(col_n.to_string());
				});
			}
		}).body(|body| {
			body.rows(text_height, st_header.entry_count as usize, |mut row| {
				let row_n = row.index();
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
