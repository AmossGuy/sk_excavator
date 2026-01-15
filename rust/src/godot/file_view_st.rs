use godot::prelude::*;
use godot::classes::{Tree, TreeItem};

use std::error::Error;
use std::io::Cursor;

use crate::filesystem::cruddy_complex_load;
use crate::formats::st::read_st;
use crate::godot::browser_tree::ItemSource;

#[derive(GodotClass)]
#[class(init, base=Tree)]
pub struct FileViewSt {
	base: Base<Tree>,
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
		
		let chunks = stuff.strings.chunks(field_count);
		for (i, chunk) in chunks.enumerate() {
			if i == 0 {
				for (j, column_name) in chunk.iter().enumerate() {
					self.base_mut().set_column_title(j as i32, column_name);
				}
			} else {
				self.base_mut().create_item().unwrap();
			}
		}
		Ok(())
	}
}
