use crate::{core::app::ExcavatorContext, file_view::{FileView, parse_as_format}};
use excavator_backend::formats::FileFormat;
use std::{fs, path::PathBuf, thread};

pub fn spawn_load_thread(file_path: PathBuf, excavator: &ExcavatorContext) {
	let excavator = excavator.clone();
	thread::spawn(move || {
		let _on_panic = OnPanic::new(|| {
			excavator.set_file_view(error_view(anyhow::anyhow!("load thread panicked")));
		});
		
		match do_load(file_path, &excavator) {
			Ok(view) => excavator.set_file_view(view),
			Err(e) => excavator.set_file_view(error_view(e)),
		}
	});
}

fn do_load(file_path: PathBuf, excavator: &ExcavatorContext) -> anyhow::Result<Box<dyn FileView>> {
	excavator.set_file_view(status_view("Reading file"));
	let file_contents = fs::read(&file_path)?;
	
	excavator.set_file_view(status_view("Parsing file"));
	let format = FileFormat::from_path(file_path);
	let view = parse_as_format(file_contents, format)?;
	
	Ok(view)
}

fn status_view(text: impl Into<String>) -> Box<dyn FileView> {
	Box::new(LoadingFileView { text: text.into() })
}

fn error_view(error: anyhow::Error) -> Box<dyn FileView> {
	Box::new(LoadErrorFileView { error })
}

struct LoadingFileView {
	text: String,
}

impl FileView for LoadingFileView {
	fn ui(&mut self, ui: &mut egui::Ui, _excavator: &ExcavatorContext) {
		ui.horizontal(|ui| {
			ui.spinner();
			ui.label(&self.text);
		});
	}
}

struct LoadErrorFileView {
	error: anyhow::Error,
}

impl FileView for LoadErrorFileView {
	fn ui(&mut self, ui: &mut egui::Ui, _excavator: &crate::core::app::ExcavatorContext) {
		let error_fg_color = ui.visuals().error_fg_color;
		ui.colored_label(error_fg_color, format!("An error occured while loading:\n{}", self.error));
	}
}

struct OnPanic<F: FnOnce()> {
	func: Option<F>,
}

impl<F: FnOnce()> OnPanic<F> {
	fn new(on_panic: F) -> Self {
		Self { func: Some(on_panic) }
	}
}

impl<F: FnOnce()> Drop for OnPanic<F> {
	fn drop(&mut self) {
		if thread::panicking() && let Some(func) = self.func.take() {
			func();
		}
	}
}
