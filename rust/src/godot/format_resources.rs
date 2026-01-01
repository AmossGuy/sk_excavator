use godot::prelude::*;
use godot::classes::file_access::ModeFlags;

#[derive(GodotClass)]
#[class(no_init, base=Resource)]
pub struct SkePak {
	#[var(get)]
	file_names: PackedArray<GString>,
	base: Base<Resource>,
}

#[godot_api]
impl SkePak {
	#[func]
	pub fn open_file(path: GString) -> Gd<Self> {
		Gd::from_init_fn(|base| {
			let mut reader = GFile::open(&path, ModeFlags::READ).unwrap();
			let index = crate::formats::pak::PakIndex::create_index(&mut reader).unwrap();
			let file_names = index.files.into_iter().map(|f| {
				GString::try_from_cstr(&f.0, Encoding::Utf8).unwrap()
			}).collect();
			
			Self {
				file_names,
				base,
			}
		})
	}
}
