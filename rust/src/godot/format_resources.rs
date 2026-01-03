use godot::prelude::*;
use godot::classes::file_access::ModeFlags;

use crate::formats::pak::{self, PakIndex};

#[derive(GodotClass)]
#[class(no_init, base=Resource)]
pub struct SkePak {
	file: GFile,
	index: PakIndex,
	base: Base<Resource>,
}

#[godot_api]
impl SkePak {
	#[func]
	pub fn open_file(path: GString) -> Gd<Self> {
		Gd::from_init_fn(|base| {
			let mut file = GFile::open(&path, ModeFlags::READ).unwrap();
			let index = PakIndex::create_index(&mut file).unwrap();
			
			Self { file, index, base }
		})
	}
	
	#[func]
	pub fn get_file_names(&self) -> PackedArray<GString> {
		self.index.files.iter().map(|f| {
			GString::try_from_cstr(&f.0, Encoding::Utf8).unwrap()
		}).collect()
	}
	
	#[func]
	pub fn read_archived_file(&mut self, name: GString) -> PackedArray<u8> {
		let file_entry = self.index.files.iter().find(|entry| {
			GString::try_from_cstr(&entry.0, Encoding::Utf8).unwrap() == name
		}).unwrap();
		let data = pak::read_whole_file(&file_entry.1, &mut self.file).unwrap();
		data.as_slice().into()
	}
}
