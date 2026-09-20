pub mod anb;
mod common;
pub mod pak;

use crate::app::context::ExcavatorContext;
use excavator_backend::formats::FileFormat;

use egui::Ui;
use std::borrow::Cow;

#[allow(unused_variables)]
pub trait FileView: Send + 'static {
	fn ui(&mut self, ui: &mut Ui, excavator: &ExcavatorContext);
	
	fn action_execute(&mut self, action: FileViewAction, excavator: &ExcavatorContext) {}
	fn action_should_be_enabled(&self, action: FileViewAction) -> bool { false }
	
	fn undo_history(&self) -> Option<Vec<Cow<'static, str>>> { None }
	fn undo_history_index(&self) -> Option<usize> { None }
	fn undo_go_to_index(&mut self, index: usize) {}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FileViewAction {
	Save,
	SaveAs,
	Undo,
	Redo,
}

pub fn parse_as_format(file_contents: Vec<u8>, format: Option<FileFormat>) -> anyhow::Result<Box<dyn FileView>> {
	let view: Box<dyn FileView> = match format {
		Some(FileFormat::Pak) => Box::new(pak::parse_pak(file_contents)?),
		Some(FileFormat::Anb) => Box::new(anb::parse_anb(file_contents)?),
		Some(_) | None => anyhow::bail!("unsupported format"),
	};
	Ok(view)
}
