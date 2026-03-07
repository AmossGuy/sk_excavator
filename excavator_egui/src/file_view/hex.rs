use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::file_view::FileBytes;
use excavator_formats::util_binary::{ParserReflect, ParserReflectContext};

pub fn hexedit_ui(bytes: &FileBytes, parse: Option<&dyn ParserReflect>, ui: &mut Ui) {
	let slice = bytes.as_slice();
	
	if let Some(parse) = parse {
		ui.label(format!("root: {:?}", parse));
		parse.get_subordinates(&mut ParserReflectContext::new(slice, &mut |subord| {
			ui.label(format!("direct subord: {:?}", subord));
		}));
	}
	
	let column_count = 0x10;
	let text_height = egui::TextStyle::Body
		.resolve(ui.style())
		.size.max(ui.spacing().interact_size.y);
	
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
