#![forbid(unsafe_code)]

mod core;
mod file_tree;
mod file_view;

fn main() -> eframe::Result {
	crate::core::app::ExcavatorApp::main()
}
