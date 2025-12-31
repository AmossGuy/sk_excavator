pub mod level;
pub mod pak;
pub mod st;
mod util_binary;

use std::ffi::OsStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileType {
	Unknown,
	Pak,
	StmOrStb,
	Stl,
}

impl FileType {
	pub fn from_extension(ext: Option<&OsStr>) -> Self {
		match ext.map(|e| e.as_encoded_bytes()) {
			Some(b"pak") => Self::Pak,
			Some(b"stm" | b"stb") => Self::StmOrStb,
			Some(b"stl") => Self::Stl,
			_ => Self::Unknown,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;
	
	#[test]
	fn known_extensions() {
		let examples = vec![
			("cool/file.pak", FileType::Pak),
			("cool/file.stb", FileType::StmOrStb),
			("cool/file.stm", FileType::StmOrStb),
			("cool/file.stl", FileType::Stl),
		];
		
		for example in examples {
			let extension = Path::new(example.0).extension();
			assert_eq!(FileType::from_extension(extension), example.1);
		}
	}
	
	#[test]
	fn unknown_extensions() {
		let examples = vec![
			"cool/file",
			"file.lfdlfkdjflkdf",
			"file.blp",
		];
		
		for example in examples {
			let extension = Path::new(example).extension();
			assert_eq!(FileType::from_extension(extension), FileType::Unknown);
		}
	}
}
