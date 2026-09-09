#![forbid(unsafe_code)]

mod app;
mod file_view;

fn main() -> eframe::Result {
	crate::app::context::ExcavatorApp::main()
}
