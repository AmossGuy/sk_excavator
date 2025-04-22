mod pak;

use std::fs::File;
use std::io::BufReader;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
	#[arg(value_name = "FILE")]
	filename: String,
}

fn main() -> std::io::Result<()> {
	let args = Args::parse();
	
	let mut reader = BufReader::new(File::open(args.filename)?);
	
	let index = pak::PakIndex::create_index(&mut reader)?;
	println!("{:?}", index);
	
	Ok(())
}
