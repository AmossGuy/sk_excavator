pub mod anb;
pub mod ltb;
pub mod pak;
pub mod st;
mod wflz;

use image::ImageFormat;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FileFormat {
	// archive
	Pak,
	
	// tabular data
	Stb,
	Stl,
	Stm,
	
	// graphics
	Anb,
	Image(ImageFormat),
	
	// level data
	Ltb,
	Lvb,
}

impl FileFormat {
	pub fn from_filename<T: AsRef<[u8]>>(path: T) -> Option<Self> {
		let filename = path.as_ref().split(|b| *b == b'/' || *b == b'\\').last().unwrap_or_default();
		let extension = filename.split(|b| *b == b'.').last().unwrap_or_default();
		Self::from_extension(extension)
	}
	
	// Could be pub, but I can't think of a reason to use it outside of from_filename
	fn from_extension<T: AsRef<[u8]>>(ext: T) -> Option<Self> {
		let ext = ext.as_ref();
		match ext.to_ascii_lowercase().as_slice() {
			b"pak" => Some(Self::Pak),
			b"stb" => Some(Self::Stb),
			b"stl" => Some(Self::Stl),
			b"stm" => Some(Self::Stm),
			b"anb" => Some(Self::Anb),
			b"ltb" => Some(Self::Ltb),
			b"lvb" => Some(Self::Lvb),
			
			// If it isn't one of the extensions above, see whether it's one of the extensions the image crate knows.
			// We only return None if ImageFormat::from_extension does.
			_ => str::from_utf8(ext).ok()
				.and_then(|ext| ImageFormat::from_extension(ext))
				.map(|format| Self::Image(format)),
		}
	}
}

pub trait EditableData {
	fn struct_name(&self) -> &str;
	fn display(&mut self, renderer: impl EditableDataRenderer);
}

pub trait EditableDataRenderer {
	type Dropdown<'a>: DropdownRenderer;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(Self::Dropdown<'_>));
	fn field_f32(&mut self, name: &str, value: &mut f32);
	fn field_u32(&mut self, name: &str, value: &mut u32);
	fn field_vec_u8(&mut self, name: &str, value: &mut Vec<u8>);
}

pub trait DropdownRenderer {
	fn choice(&mut self, name: &str, selected: bool) -> bool;
}

pub trait FieldDispatch {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str);
}

impl FieldDispatch for f32 {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_f32(name, self);
	}
}

impl FieldDispatch for u32 {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_u32(name, self);
	}
}

impl FieldDispatch for Vec<u8> {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_vec_u8(name, self);
	}
}
