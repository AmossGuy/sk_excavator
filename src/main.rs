pub mod formats;
pub mod interface;

fn main() -> binrw::BinResult<()> {
	crate::interface::cli::cli_main()
}
