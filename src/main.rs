mod cli;
mod loctext;
mod pak;
mod util_binary;

fn main() -> std::io::Result<()> {
	cli::cli_main()
}
