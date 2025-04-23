mod cli;
mod loctext;
mod pak;

fn main() -> std::io::Result<()> {
	cli::cli_main()
}
