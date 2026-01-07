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
		self.base_mut().set_hide_root(true);
		self.base_mut().set_column_titles_visible(true);
		
		let data = cruddy_complex_load(source)?;
		let stuff = read_st(&mut Cursor::new(data), true)?;
		let field_count = stuff.field_count;
		let mut column = 0;
		let _root = self.base_mut().create_item().unwrap();
		let mut item: Option<Gd<TreeItem>> = None;
		for string in &stuff.strings {
			if item.is_none() {
				self.base_mut().set_column_title(column, string);
			} else {
				item.as_mut().unwrap().set_text(column, string);
			}
			
			if column < (field_count - 1).try_into().unwrap() {
				column += 1;
			} else {
				column = 0;
				item = Some(self.base_mut().create_item().unwrap());
			}
		}
		Ok(())
	}
}
