#![forbid(unsafe_code)]

mod core;
mod file_view;
mod misc;

fn main() -> eframe::Result {
	crate::core::app::ExcavatorApp::main()
}
