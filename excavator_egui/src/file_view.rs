pub mod anb;
mod common;
pub mod pak;

use crate::core::app::ExcavatorContext;
use crate::core::menubar::ViewAction;
use excavator_backend::formats::FileFormat;
use egui::Ui;

pub trait FileView: Send + Sync + 'static {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext);
	
	fn menubar_execute(&mut self, action: ViewAction) {
		let _ = action;
		// Nothing to be done in the default implementation...
	}
	
	fn menubar_should_be_enabled(&self, action: ViewAction) -> bool {
		let _ = action;
		false
	}
}

pub fn parse_as_format(file_contents: Vec<u8>, format: Option<FileFormat>) -> anyhow::Result<Box<dyn FileView>> {
	let view: Box<dyn FileView> = match format {
		Some(FileFormat::Pak) => Box::new(pak::parse_pak(file_contents)?),
		Some(FileFormat::Anb) => Box::new(anb::parse_anb(file_contents)?),
		Some(_) | None => anyhow::bail!("unsupported format"),
	};
	Ok(view)
}
