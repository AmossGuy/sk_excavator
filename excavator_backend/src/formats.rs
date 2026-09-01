#![expect(unused)] // in progress

pub mod anb;
pub mod common;
pub mod pak;
pub mod wflz;

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
			_ => None,
		}
	}
}
