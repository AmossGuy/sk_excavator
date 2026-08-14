pub mod anb;
mod common;
pub mod image;
pub mod loader;
pub mod ltb;
pub mod pak;
pub mod st;

pub use loader::FileViewLoader;

trait FileView: Send {
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect;
}

impl<T, E> FileView for Result<T, E> where
	T: FileView + Send,
	E: ToString + Send,
{
	fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		match self {
			Ok(file_view) => file_view.ui(ui),
			Err(e) => {
				ui.label("Error");
				ui.label(e.to_string());
				FileViewEffect::default()
			},
		}
	}
}

#[derive(Default)]
pub struct FileViewEffect {
}
