use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::formats::{pak, st, FileType};

#[derive(Debug, Parser)]
#[command(about = "Performs various operations on Shovel Knight's data files")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	#[command(visible_alias = "l", about = "List the contents of one or more .pak files")]
	List {
		#[arg(num_args = 1.., required = true, value_name = "PAK", help = "Path to the .pak file(s)")]
		pak_paths: Vec<PathBuf>,
	},
	#[command(visible_alias = "x", about = "Extract the contents of one or more .pak files")]
	Extract {
		#[arg(num_args = 1.., required = true, value_name = "PAK", help = "Path(s) to the .pak file(s)")]
		pak_paths: Vec<PathBuf>,
		#[arg(short = 'd', long = "dest", value_name = "DEST", help = "Directory to place the extracted files - omit to use working directory")]
		dest_path: Option<PathBuf>,
	},
	#[command(visible_alias = "st", about = "Convert .stb, .stl, or .stm files into a readable JSON format")]
	StTemporary {
	},
}

/*
fn find_common_location(paths: &Vec<PathBuf>) -> Option<String> {
	// When there's only one file, it would be pointless to separate out its location
	// And zero-length vec would cause a panic in following code
	if paths.len() == 0 || paths.len() == 1 {
		return None;
	}
	
	// The main point of this: if all the files are in the same directory, show that directory's path separately to avoid repeating it
	let first_location = paths[0].parent()?;
	for path in paths.iter().skip(1) {
		if path.parent() != Some(first_location) {
			return None;
		}
	}
	
	// A empty path would be invisible in the output
	// I believe the path canonicalization prevents this from actually occurring, but it doesn't hurt
	if first_location.as_os_str().is_empty() {
		return None;
	} else {
		return Some(first_location.to_string_lossy().to_string());
	}
}

fn process_file_paths(paths: Vec<PathBuf>) -> io::Result<Vec<(PathBuf, String)>> {
	let canon_paths = paths.into_iter().map(|p| p.canonicalize()).collect::<io::Result<_>>()?;
	
	let common_location = find_common_location(&canon_paths);
	
	Ok(canon_paths.into_iter().map(|path| {
		let display = match common_location {
			Some(_) => path.file_name().unwrap_or(OsStr::new("")),
			None => OsStr::new(&path),
		}.to_string_lossy().to_string();
		(path, display)
	}).collect())
}
*/

pub fn cli_main() -> binrw::BinResult<()> {
	let args = Cli::parse();
	
	match args.command {
		/*
		Commands::List { file_paths } => {
			let file_paths = process_file_paths(file_paths)?;
			for (path, display) in file_paths {
				println!("= FILE: {} =", display);
				match FileType::from_extension(path.extension()) {
					FileType::Unknown => println!("Unknown file format"),
					FileType::Pak => {
						todo!();
					},
					FileType::Stb => {
						todo!();
					},
				}
			}
			todo!();
		},
		Commands::Extract { file_paths, dest_path } => {
			todo!();
		},
		*/
		Commands::List { pak_paths } => {
			if !pak_paths.iter().all(|x| FileType::from_extension(x.extension()) == FileType::Pak) {
				panic!("wtf bro");
			}
			
			for pak_path in pak_paths {
				println!("FILE: {}", pak_path.display());
				
				let mut reader = BufReader::new(File::open(pak_path)?);
				let index = pak::PakIndex::create_index(&mut reader)?;
				for entry in index.files {
					let name = String::from_utf8_lossy(entry.0.to_bytes());
					println!("{} ({} bytes)", name, entry.1.data_length);
				}
			}
		},
		Commands::Extract { pak_paths, dest_path } => {
			if !pak_paths.iter().all(|x| FileType::from_extension(x.extension()) == FileType::Pak) {
				panic!("wtf bro");
			}
			
			for pak_path in pak_paths {
				println!("FILE: {}", pak_path.display());
				
				let dest_path = match dest_path {
					Some(ref string) => PathBuf::from(string),
					None => std::env::current_dir()?,
				};
				
				let mut reader = BufReader::new(File::open(pak_path)?);
				let index = pak::PakIndex::create_index(&mut reader)?;
				for entry in index.files {
					let name = String::from_utf8_lossy(entry.0.to_bytes());
					println!("Extracting {}...", name);
					let looksee = pak::read_whole_file(&entry.1, &mut reader)?;
					let save_location = dest_path.join(name.to_string());
					if let Some(save_dir) = save_location.parent() {
						std::fs::create_dir_all(save_dir)?;
					}
					let mut output_file = File::create(save_location)?;
					output_file.write_all(&looksee)?;
				}
				println!("Done.");
			}
		},
		Commands::StTemporary {} => {
			#[allow(deprecated)] // this code is temporary and not under stress
			let home = std::env::home_dir().unwrap();
			let location = home.join("Documents/shovel-knight-rip-testing");
			
			for file in ["loctext/dialogue.stm", "global_free.pak/dialogue/speakers.stb", "loctext/dialogue_eng.stl"] {
				println!("FILE: {}", file);
				let path = location.join(file);
				let file_type = FileType::from_extension(path.extension());
				let stl = file_type == FileType::Stl;
				let mut reader = BufReader::new(File::open(path)?);
				st::read_st_wip(&mut reader, stl)?;
			}
		},
	}
	
	Ok(())
}
