pub mod anb;
mod common;
pub mod pak;

use crate::core::app::ExcavatorContext;
use excavator_backend::formats::FileFormat;
use egui::Ui;

pub trait FileView: Send + Sync + 'static {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext);
}

pub fn parse_as_format(file_contents: Vec<u8>, format: Option<FileFormat>) -> anyhow::Result<Box<dyn FileView>> {
	let view: Box<dyn FileView> = match format {
		Some(FileFormat::Pak) => Box::new(pak::PakFileView::parse(file_contents)?),
		Some(FileFormat::Anb) => Box::new(anb::AnbFileView::parse(file_contents)?),
		Some(_) | None => anyhow::bail!("unsupported format"),
	};
	Ok(view)
}
