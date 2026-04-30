#![forbid(unsafe_code)]

mod core;
mod file_tree;
mod file_view;
mod misc;

static EXECUTOR: async_executor::StaticExecutor = async_executor::StaticExecutor::new();

fn main() -> eframe::Result {
	std::thread::spawn(|| {
		loop {
			futures_lite::future::block_on(EXECUTOR.tick());
		}
	});
	
	crate::core::app::ExcavatorApp::main()
}
