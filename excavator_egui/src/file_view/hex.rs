use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::file_view::FileBytes;
use excavator_backend::formats::binary::{ParserReflect, ParserReflectContext, ParserStructError, StructRole};
use excavator_backend::parse::{FullParseLogger, ParseLogger};
use excavator_backend::rust_lapper::Lapper;

pub type ParserReflectMaker = fn(&[u8]) -> Option<&dyn ParserReflect>;

pub struct HexFileView {
	bytes: FileBytes,
	analysis: TempAnalysisInfo,
	dumb_scroll_offset: egui::Vec2,
	clicked_address: Option<usize>,
	struct_debug: Option<String>,
}

// only to keep the outdated code i haven't QUITE gotten around to replacing working
// it'll disappear once anb's updated to the new system
pub enum TempAnalysisInfo {
	None,
	Old(ParserReflectMaker),
	New(Lapper<u64, ()>),
}

impl HexFileView {
	pub fn new(bytes: FileBytes, analysis: TempAnalysisInfo) -> Self {
		Self {
			bytes, analysis,
			dumb_scroll_offset: egui::Vec2::default(),
			clicked_address: None,
			struct_debug: None,
		}
	}
	
	pub fn ui(&mut self, ui: &mut Ui) {
		if let Some(clicked_address) = &self.clicked_address {
			ui.label(format!("Selected address: 0x{:X}", clicked_address));
		}
		
		if let Some(struct_debug) = &self.struct_debug {
			ui.label(struct_debug);
		}
		
		let slice = self.bytes.as_slice();
		
		let column_count = 0x10;
		let text_height = egui::TextStyle::Body
			.resolve(ui.style())
			.size.max(ui.spacing().interact_size.y);
		
		let mut table = TableBuilder::new(ui);
		table = table.striped(false);
		
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
		
		let highlighter = HighlightRenderer::new(&painter, HighlightSettings {
			grid_topleft: ui_cursor.min - self.dumb_scroll_offset,
			grid_cell_size: egui::Vec2::new(available_width / column_count as f32, text_height + item_spacing.y),
			column_count,
		});
		
		if let Some(clicked_address) = self.clicked_address {
			highlighter.highlight_range(clicked_address, 1, ui.visuals().selection.bg_fill);
		}
		
		if !(matches!(self.analysis, TempAnalysisInfo::None)) {
			let clicked_address = self.clicked_address;
			
			self.struct_debug = None;
			let struct_debug_cell = std::cell::RefCell::new(&mut self.struct_debug);
			
			let highlight_struct = |r#struct: &dyn ParserReflect, color: egui::Color32| {
				let r#struct: &dyn std::fmt::Debug = r#struct;
				
				let start = std::ptr::from_ref(r#struct).addr() - slice.as_ptr().addr();
				let length = std::mem::size_of_val(r#struct);
				
				highlighter.highlight_range(start, length, color);
				
				if clicked_address.is_some_and(|a| (start..start+length).contains(&a)) {
					**struct_debug_cell.borrow_mut() = Some(format!("{:?}", r#struct));
				}
			};
			
			let i = std::cell::RefCell::<usize>::new(0);
			
			let mut closure_1 = |struct_s: Result<&dyn ParserReflect, ParserStructError>| {
				if let Ok(struct_s) = struct_s {
					let color = whoa_colors(struct_s.role(), *i.borrow());
					highlight_struct(struct_s, color);
					*i.borrow_mut() += 1;
				}
			};
			
			let mut closure_2 = |slice_s: Result<&[u8], ParserStructError>| {
				if let Ok(slice_s) = slice_s {
					let start = slice_s.as_ptr().addr() - slice.as_ptr().addr();
					let length = slice_s.len();
					let color = whoa_colors(StructRole::CompressionLiterals, *i.borrow());
					
					highlighter.highlight_range(start, length, color);
					if clicked_address.is_some_and(|a| (start..start+length).contains(&a)) {
						**struct_debug_cell.borrow_mut() = Some(format!("{:?}", slice_s));
					}
					
					*i.borrow_mut() += 1;
				}
			};
			
			if let TempAnalysisInfo::Old(f) = &self.analysis {
				if let Some(parse) = f(slice) {
					let mut reflector = ParserReflectContext::new(slice, &mut closure_1, &mut closure_2);
					reflector.ingest2_dyn(Ok(parse))
				}
			} else if let TempAnalysisInfo::New(lapper) = &self.analysis {
				// Why am I even using Lapper if I can't be arsed to write code that does the thing I wanted to use it for!?
				for interval in lapper.iter() {
					highlighter.highlight_range(interval.start as usize, (interval.stop - interval.start) as usize, egui::Color32::DARK_GRAY.gamma_multiply(0.4))
				}
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

fn whoa_colors(role: StructRole, i: usize) -> egui::Color32 {
	let (color, zebra) = match role {
		StructRole::CompressionBlock => (egui::Color32::DARK_GREEN, false),
		StructRole::CompressionLiterals => (egui::Color32::ORANGE, false),
		_ => (egui::Color32::DARK_GRAY, /* true */ false), // zebra looking good would need structs to be sorted beforehand
	};
	
	if zebra {
		color.gamma_multiply([0.3, 0.5][i % 2])
	} else {
		color.gamma_multiply(0.4)
	}
}

struct HighlightSettings {
	grid_topleft: egui::Pos2,
	grid_cell_size: egui::Vec2,
	column_count: usize,
}

struct HighlightRenderer<'a> {
	painter: &'a egui::Painter,
	settings: HighlightSettings,
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
		
		let mut rect = egui::Rect::from_min_size(
			grid_topleft + egui::Vec2::new((segment.start % column_count) as f32 * grid_cell_size.x, (segment.start / column_count) as f32 * grid_cell_size.y),
			egui::Vec2::new(grid_cell_size.x * segment.length as f32, grid_cell_size.y),
		);
		
		if segment.start_cap {
			rect.min.x += 2.0;
		}
		if segment.end_cap {
			rect.max.x -= 2.0;
		}
		
		self.painter.rect_filled(
			rect,
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
