use egui::Ui;
use egui_extras::{Column, TableBuilder};
use std::collections::VecDeque;

use crate::file_view::FileBytes;
use excavator_formats::util_binary::{ParserReflect, ParserReflectContext};

pub type ParserReflectMaker = fn(&[u8]) -> Option<&dyn ParserReflect>;

pub struct HexFileView {
	bytes: FileBytes,
	reflect_maker: Option<ParserReflectMaker>,
	dumb_scroll_offset: egui::Vec2,
	clicked_address: Option<usize>,
	struct_debug: Option<String>,
}

impl HexFileView {
	pub fn new(bytes: FileBytes, reflect_maker: Option<ParserReflectMaker>) -> Self {
		Self {
			bytes, reflect_maker,
			dumb_scroll_offset: egui::Vec2::default(),
			clicked_address: None,
			struct_debug: None,
		}
	}
	
	pub fn ui(&mut self, ui: &mut Ui) {
		if let Some(struct_debug) = &self.struct_debug {
			ui.label(struct_debug);
		}
		
		let slice = self.bytes.as_slice();
		let parse = self.reflect_maker.and_then(|f| f(slice));
		
		let column_count = 0x10;
		let text_height = egui::TextStyle::Body
			.resolve(ui.style())
			.size.max(ui.spacing().interact_size.y);
		
		let mut table = TableBuilder::new(ui);
		table = table.striped(true);
		
		for _ in 0..column_count {
			table = table.column(Column::remainder().clip(true));
		}
		
		let mut table = table.header(20.0, |mut table_header| {
			for col_n in 0..column_count {
				table_header.col(|ui| {
					ui.strong(format!("{:X}", col_n));
				});
			}
		});
		
		let ui = table.ui_mut();
		let available_width = ui.available_width();
		let item_spacing = ui.spacing().item_spacing;
		let ui_cursor = ui.cursor();
		let painter = ui.painter().with_clip_rect(ui_cursor);
		
		if let Some(parse) = parse {
			let highlighter = HighlightRenderer::new(&painter, HighlightSettings {
				grid_topleft: ui_cursor.min - self.dumb_scroll_offset,
				grid_cell_size: egui::Vec2::new(available_width / column_count as f32, text_height + item_spacing.y),
				column_count,
			});
			
			let clicked_address = self.clicked_address;
			self.struct_debug = None;
			
			let struct_debug_cell = std::cell::RefCell::new(&mut self.struct_debug);
			
			let highlight_struct = |r#struct, color: egui::Color32| {
				let r#struct: &dyn std::fmt::Debug = r#struct;
				
				let start = std::ptr::from_ref(r#struct).addr() - slice.as_ptr().addr();
				let length = std::mem::size_of_val(r#struct);
				
				highlighter.highlight_range(start, length, color.gamma_multiply(0.4));
				
				if clicked_address.is_some_and(|a| (start..start+length).contains(&a)) {
					**struct_debug_cell.borrow_mut() = Some(format!("{:?}", r#struct));
				}
			};
			
			let mut structs_to_highlight = VecDeque::from([parse]);
			let mut i: usize = 0;
			
			while let Some(current_struct) = structs_to_highlight.pop_front() {
				let colors = [egui::Color32::DARK_RED, egui::Color32::DARK_GREEN, egui::Color32::DARK_BLUE, egui::Color32::ORANGE];
				let color = colors[i % colors.len()];
				highlight_struct(current_struct, color);
				
				current_struct.get_subordinates(&mut ParserReflectContext::new(slice, &mut |subord| {
					if let Ok(subord) = subord {
						structs_to_highlight.push_back(subord);
					}
				}, &mut |slice_s| {
					if let Ok(slice_s) = slice_s {
						i += 1;
						let start = slice_s.as_ptr().addr() - slice.as_ptr().addr();
						let length = slice_s.len();
						let color = colors[i % colors.len()];
						
						highlighter.highlight_range(start, length, color);
						if clicked_address.is_some_and(|a| (start..start+length).contains(&a)) {
							**struct_debug_cell.borrow_mut() = Some(format!("{:?}", slice_s));
						}
					}
				}));
				
				i += 1;
			}
		}
		
		let scroll_output = table.body(|body| {
			body.rows(text_height, slice.len() / column_count, |mut row| {
				let row_start = row.index() * column_count;
				let subslice = slice.get(row_start..(row_start + column_count)).unwrap_or_default();
				for (column_index, byte) in subslice.iter().enumerate() {
					row.col(|ui| {
						ui.label(format!("{:02X}", byte));
						
						let clicked = ui.interact(
							egui::Rect::EVERYTHING,
							ui.id().with("hexedit click"),
							egui::Sense::CLICK,
						).clicked();
						if clicked {
							self.clicked_address = Some(row_start + column_index);
						}
					});
				}
			});
		});
		
		self.dumb_scroll_offset = scroll_output.state.offset;
	}
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
			
			let is_last_segment = end <= next_row_start;
			
			self.draw_segment(&HighlightSegment {
				start: cursor,
				length: std::cmp::min(end, next_row_start) - cursor,
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
