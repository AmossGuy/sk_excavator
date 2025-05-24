pub mod formats;
pub mod interface;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	crate::interface::cli::cli_main()
}
