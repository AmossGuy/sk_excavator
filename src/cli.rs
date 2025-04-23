use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::pak::{self, PakIndex};

#[derive(Debug, Parser)]
#[command(about = "Performs various operations on Shovel Knight's data files")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	#[command(arg_required_else_help = true, visible_alias = "l", about = "List the contents of a .pak file")]
	List {
		#[arg(value_name = "PAK", help = "Path to the .pak file")]
		pak_path: String,
	},
	#[command(arg_required_else_help = true, visible_alias = "x", about = "Extract the contents of a .pak file")]
	Extract {
		#[arg(value_name = "PAK", help = "Path to the .pak file")]
		pak_path: String,
		#[arg(value_name = "DEST", help = "Directory to extract files to - omit to use working directory")]
		dest_path: Option<String>,
	},
}

pub fn cli_main() -> std::io::Result<()> {
	let args = Cli::parse();
	
	match args.command {
		Commands::List { pak_path } => {
			let mut reader = BufReader::new(File::open(pak_path)?);
			let index = PakIndex::create_index(&mut reader)?;
			for entry in index.files {
				let name = String::from_utf8_lossy(entry.0.to_bytes());
				println!("{} ({} bytes)", name, entry.1.data_length);
			}
		},
		Commands::Extract { pak_path, dest_path } => {
			let dest_path = match dest_path {
				Some(string) => PathBuf::from(string),
				None => std::env::current_dir()?,
			};
			
			let mut reader = BufReader::new(File::open(pak_path)?);
			let index = PakIndex::create_index(&mut reader)?;
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
		},
	}
	
	Ok(())
}
