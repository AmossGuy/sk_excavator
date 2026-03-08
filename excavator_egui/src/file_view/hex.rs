use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::file_view::FileBytes;
use excavator_formats::util_binary::{ParserReflect, ParserReflectContext};

pub fn hexedit_ui(bytes: &FileBytes, parse: Option<&dyn ParserReflect>, ui: &mut Ui) {
	let slice = bytes.as_slice();
	
	/*
	if let Some(parse) = parse {
		ui.label(format!("root: {:?}", parse));
		parse.get_subordinates(&mut ParserReflectContext::new(slice, &mut |subord| {
			ui.label(format!("direct subord: {:?}", subord));
		}));
	}
	*/
	
	let column_count = 0x10;
	let text_height = egui::TextStyle::Body
		.resolve(ui.style())
		.size.max(ui.spacing().interact_size.y);
	
	let painter = ui.painter();
	let highlighter = HighlightRenderer::new(painter, HighlightSettings {
		grid_topleft: painter.clip_rect().min,
		grid_cell_size: egui::Vec2::new(30.0, 30.0), // placeholder
		column_count,
	});
	
	for i in 0..4usize {
		let colors = [egui::Color32::DARK_RED, egui::Color32::DARK_GREEN, egui::Color32::DARK_BLUE, egui::Color32::DARK_GRAY];
		let color = colors[i % colors.len()];
		highlighter.highlight_range(i * 5, 5, color);
	}
	
	/*
	// painter test
	painter.rect_filled(
		egui::Rect::from_min_size(painter.clip_rect().min, egui::Vec2::new(100.0, 200.0)),
		egui::CornerRadius::same(5),
		egui::Color32::KHAKI,
	);
	*/
	
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

struct HighlightSettings {
	grid_topleft: egui::Pos2,
	grid_cell_size: egui::Vec2,
	column_count: usize,
}

struct HighlightRenderer<'a> {
	painter: &'a egui::Painter,
	settings: HighlightSettings
}

impl<'a> HighlightRenderer<'a> {
	fn new(painter: &'a egui::Painter, settings: HighlightSettings) -> Self {
		Self { painter, settings }
	}
	
	// Basically: render a rectangle for each line that contains part of the highlighted range
	fn highlight_range(&self, start: usize, length: usize, color: egui::Color32) {
		let column_count = self.settings.column_count;
		let end = start.saturating_add(length);
		let mut cursor = start;
		
		let mut is_first_segment = true;
		loop {
			let next_row_start = if cursor.is_multiple_of(column_count) {
				cursor + column_count
			} else {
				cursor.next_multiple_of(column_count)
			};
			
			let is_last_segment = end < next_row_start;
			
			self.draw_segment(&HighlightSegment {
				start: cursor,
				length: std::cmp::min(end, next_row_start) - start,
				start_cap: is_first_segment,
				end_cap: is_last_segment,
			}, color);
			
			if is_last_segment {
				break;
			}
			
			cursor = next_row_start;
			is_first_segment = false;
		}
	}
	
	fn draw_segment(&self, segment: &HighlightSegment, color: egui::Color32) {
		let corner_radius = 5;
		
		let grid_topleft = self.settings.grid_topleft;
		let grid_cell_size = self.settings.grid_cell_size;
		let column_count = self.settings.column_count;
		
		self.painter.rect_filled(
			egui::Rect::from_min_size(
				grid_topleft + egui::Vec2::new((segment.start % column_count) as f32 * grid_cell_size.x, (segment.start / column_count) as f32 * grid_cell_size.y),
				egui::Vec2::new(grid_cell_size.x * segment.length as f32, grid_cell_size.y),
			),
			egui::CornerRadius {
				nw: if segment.start_cap { corner_radius } else { 0 },
				ne: if segment.end_cap { corner_radius } else { 0 },
				sw: if segment.start_cap { corner_radius } else { 0 },
				se: if segment.end_cap { corner_radius } else { 0 },
			},
			color,
		);
	}
}

struct HighlightSegment {
	start: usize,
	length: usize,
	start_cap: bool,
	end_cap: bool,
}
