pub mod anb;
pub mod common;
pub mod pak;
pub mod wflz;

use image::ImageFormat;
use std::{ffi::OsStr, path::Path};

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
	pub fn from_path<T: AsRef<Path>>(path: T) -> Option<Self> {
		Self::from_extension(path.as_ref().extension()?)
	}
	
	// Could be pub, but I can't think of a reason to use it outside of from_filename
	fn from_extension<T: AsRef<OsStr>>(ext: T) -> Option<Self> {
		let ext = ext.as_ref();
		match ext.to_ascii_lowercase().as_encoded_bytes() {
			b"pak" => Some(Self::Pak),
			b"stb" => Some(Self::Stb),
			b"stl" => Some(Self::Stl),
			b"stm" => Some(Self::Stm),
			b"anb" => Some(Self::Anb),
			b"ltb" => Some(Self::Ltb),
			b"lvb" => Some(Self::Lvb),
			
			// If it isn't one of the extensions above, see whether it's one of the extensions the image crate knows.
			// We only return None if the image crate doesn't handle this file extension either.
			_ => {
				let ext_str = ext.to_str()?;
				let format = ImageFormat::from_extension(ext_str)?;
				Some(Self::Image(format))
			},
		}
	}
}
