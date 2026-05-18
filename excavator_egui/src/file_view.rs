pub mod anb;
pub mod image;
pub mod loader;
pub mod ltb;
pub mod st;

pub use loader::FileViewLoader;

trait FileView: Send {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect;
}

#[derive(Default)]
pub struct FileViewEffect {
}
