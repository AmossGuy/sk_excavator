pub mod loctext;
pub mod pak;
pub mod stb;
mod util_binary;

use std::ffi::OsStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileType {
	Unknown,
	Pak,
	Stb,
}

impl FileType {
	pub fn from_extension(ext: Option<&OsStr>) -> Self {
		match ext.and_then(|e| e.to_str()) {
			Some("pak") => Self::Pak,
			Some("stb" | "stl" | "stm") => Self::Stb,
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
			("cool/file.stb", FileType::Stb),
			("cool/file.stl", FileType::Stb),
			("cool/file.stm", FileType::Stb),
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
