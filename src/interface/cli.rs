use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::formats::{loctext, pak};

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
		pak_paths: Vec<String>,
	},
	#[command(visible_alias = "x", about = "Extract the contents of one or more .pak files")]
	Extract {
		#[arg(num_args = 1.., required = true, value_name = "PAK", help = "Path to the .pak file(s)")]
		pak_paths: Vec<String>,
		#[arg(short = 'd', long = "dest", value_name = "DEST", help = "Directory to extract files to - omit to use working directory")]
		dest_path: Option<String>,
	},
	#[command(about = "Extract text from a .stl file")]
	DumpLoctext {
		#[arg(value_name = "STL", help = "Path to the .stl file")]
		stl_path: String,
		#[arg(value_name = "DEST", help = "File to save the text to")]
		dest_path: String,
	},
	Ffffj {
		paths: Vec<String>,
	},
}

pub fn cli_main() -> binrw::BinResult<()> {
	let args = Cli::parse();
	
	match args.command {
		Commands::List { pak_paths } => {
			for pak_path in pak_paths {
				println!("FILE: {}", pak_path);
				
				let mut reader = BufReader::new(File::open(pak_path)?);
				let index = pak::PakIndex::create_index(&mut reader)?;
				for entry in index.files {
					let name = String::from_utf8_lossy(entry.0.to_bytes());
					println!("{} ({} bytes)", name, entry.1.data_length);
				}
			}
		},
		Commands::Extract { pak_paths, dest_path } => {
			for pak_path in pak_paths {
				println!("FILE: {}", pak_path);
				
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
		Commands::DumpLoctext { stl_path, dest_path } => {
			let mut reader = BufReader::new(File::open(stl_path)?);
			let strings = loctext::read_stl(&mut reader)?;
			
			let mut writer = File::create(dest_path)?;
			for string in strings {
				writeln!(writer, "{}", string)?;
			}
		},
		Commands::Ffffj {paths} => {
			for path in paths {
				println!("FILE: {}", path);
				let mut reader = BufReader::new(File::open(path)?);
				loctext::ffffj(&mut reader)?;
			}
		}
	}
	
	Ok(())
}
