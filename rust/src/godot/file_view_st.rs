use godot::prelude::*;
use godot::classes::{Tree, ITree};

use std::error::Error;
use std::io::Cursor;

use crate::filesystem::cruddy_complex_load;
use crate::formats::st::{read_st, StReadOutcome};
use crate::godot::browser_tree::ItemSource;

#[derive(GodotClass)]
#[class(init, base=Tree)]
pub struct FileViewSt {
	base: Base<Tree>,
	stuff: Option<StReadOutcome>,
	loaded_items: usize,
}

#[godot_api]
impl ITree for FileViewSt {
	fn process(&mut self, _delta: f64) {
		self.load_strings(10);
	}
}

impl FileViewSt {
	pub fn load_stl_stuff(&mut self, source: &ItemSource) -> Result<(), Box<dyn Error>> {
		let data = cruddy_complex_load(source)?;
		let stuff = read_st(&mut Cursor::new(data), true)?;
		let field_count = stuff.field_count;
		
		self.base_mut().set_hide_root(true);
		self.base_mut().set_column_titles_visible(true);
		self.base_mut().set_columns(field_count as i32);
		let _root = self.base_mut().create_item().unwrap();
		
		self.stuff = Some(stuff);
		Ok(())
	}
	
	fn load_strings(&mut self, amount: usize) {
		// Simple way to keep the borrow checker happy
		let Some(stuff) = self.stuff.take() else { return; };
		
		let Some(mut root) = self.base().get_root() else { return; };
		let field_count = stuff.field_count;
		
		let mut chunks = stuff.strings
			.chunks(field_count)
			.skip(self.loaded_items)
			.take(amount);
		
		if self.loaded_items == 0 {
			let chunk = chunks.next().unwrap_or_default();
			for (j, text) in chunk.into_iter().enumerate() {
				self.base_mut().set_column_title(j as i32, text);
			}
			self.loaded_items += 1;
		}
		
		for chunk in chunks {
			let mut item = root.create_child().unwrap();
			for (j, text) in chunk.into_iter().enumerate() {
				item.set_text(j as i32, text);
			}
			self.loaded_items += 1;
		}
		
		// Simple way to keep the borrow checker happy, cont
		self.stuff = Some(stuff)
	}
}
