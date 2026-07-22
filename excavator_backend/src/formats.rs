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

pub struct TreeMarker;

pub trait EditableStruct {
	fn struct_name(&self) -> &str;
	fn number_of_fields(&self) -> usize;
	fn field_name(&self, index: usize) -> Option<&str>;
	fn field_ref(&self, index: usize) -> Option<&dyn std::any::Any>;
	fn field_mut(&mut self, index: usize) -> Option<&mut dyn std::any::Any>;
}
