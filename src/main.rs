mod cli;
mod loctext;
mod pak;
mod util_binary;

fn main() -> binrw::BinResult<()> {
	cli::cli_main()
}
