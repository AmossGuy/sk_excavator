use egui::Ui;
use egui_extras::{Column, TableBuilder};
use std::io::{BufRead, Cursor, Seek};

use super::{FileView, FileViewEffect};
use excavator_backend::formats::st::{read_st_header, read_st_cell};

pub struct StFileView {
	data: anyhow::Result<Vec<u8>>,
	is_stl: bool,
}

impl StFileView {
	pub fn load_stl(reader: impl BufRead + Seek, _ctx: &egui::Context) -> Self {
		Self { data: Self::read_data(reader), is_stl: true }
	}
	
	pub fn load_not_stl(reader: impl BufRead + Seek, _ctx: &egui::Context) -> Self {
		Self { data: Self::read_data(reader), is_stl: false }
	}
	
	fn read_data(mut reader: impl BufRead + Seek) -> anyhow::Result<Vec<u8>> {
		let mut buf = Vec::new();
		reader.read_to_end(&mut buf)?;
		Ok(buf)
	}
}

impl FileView for StFileView {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		egui::ScrollArea::horizontal().show(ui, |ui| {
			self.table_ui(ui);
		});
		FileViewEffect::default()
	}
}

impl StFileView {
	fn table_ui(&mut self, ui: &mut Ui) {
		if let Err(e) = self.table_ui_inner(ui) {
			ui.label(format!("error: {}", e));
		}
	}
		
	fn table_ui_inner(&mut self, ui: &mut Ui) -> anyhow::Result<()> {
		let data = self.data.as_ref()
			.map_err(|e| anyhow::anyhow!("couldn't read data: {}", e))?;
		let mut cursor = Cursor::new(&data);
		let st_header = read_st_header(&mut cursor, self.is_stl)?;
		
		let text_height = egui::TextStyle::Body
			.resolve(ui.style())
			.size.max(ui.spacing().interact_size.y);
		
		let mut table = TableBuilder::new(ui);
		table = table.striped(true);
		
		for _ in 0..st_header.field_count {
			table = table.column(Column::remainder().clip(true));
		}
		
		fn fixify(thing: anyhow::Result<bstr::BString>) -> String {
			match thing {
				Ok(bstring) => bstring.to_string(),
				Err(e) => format!("error: {}", e),
			}
		}
		
		table.header(20.0, |mut table_header| {
			for col_n in 0..st_header.field_count as usize {
				table_header.col(|ui| {
					let cell_n = col_n;
					let string = fixify(read_st_cell(&mut cursor, &st_header, cell_n));
					ui.strong(string);
				});
			}
		}).body(|body| {
			body.rows(text_height, st_header.entry_count as usize - 1, |mut row| {
				let row_n = row.index() + 1;
				for col_n in 0..st_header.field_count as usize {
					row.col(|ui| {
						let cell_n = row_n * st_header.field_count as usize + col_n;
						let string = fixify(read_st_cell(&mut cursor, &st_header, cell_n));
						ui.label(string);
					});
				}
			});
		});
		
		Ok(())
	}
}
