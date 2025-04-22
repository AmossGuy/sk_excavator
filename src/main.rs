mod pak;

use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
	#[arg(value_name = "FILE")]
	filename: String,
	#[arg(value_name = "INNERFILE")]
	inner_filename: String,
}

fn main() -> std::io::Result<()> {
	let args = Args::parse();
	
	let mut reader = BufReader::new(File::open(args.filename)?);
	let index = pak::PakIndex::create_index(&mut reader)?;
	let entry = &index.files[&CString::new(args.inner_filename).unwrap()];
	let looksee = pak::read_whole_file(entry, &mut reader)?;
	println!("{}", String::from_utf8_lossy(&looksee));
	
	Ok(())
}
