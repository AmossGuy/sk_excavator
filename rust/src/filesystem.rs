use std::{fs, io, path::{Path, PathBuf}};

pub(crate) fn get_directory_contents(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
	let mut entries = fs::read_dir(path)?
		.map(|res| res.map(|e| e.path()))
		.collect::<Result<Vec<_>, _>>()?;
	
	entries.sort();
	Ok(entries)
}
