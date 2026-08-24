use std::io::{BufRead, BufReader, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::JoinHandle;

use excavator_backend::formats::FileFormat;
use image::ImageFormat;

use super::{FileView, FileViewEffect};

pub struct FileViewLoader {
	state: LoaderState,
}

enum LoaderState {
	Loading(JoinHandle<anyhow::Result<Box<dyn FileView>>>),
	Loaded(Box<dyn FileView>),
	Failed(Arc<anyhow::Error>),
	Placeholder,
}

impl FileViewLoader {
	pub fn from_path(path: PathBuf, ctx: &egui::Context) -> Option<Self> {
		let ctx = ctx.clone();
		match FileFormat::from_path(&path) {
			Some(FileFormat::Pak) => Some(Self::with_load_fn(move || {
				let file = open(path)?;
				let buf = std::io::BufReader::new(file);
				
				Ok(Box::new(super::pak::PakFileView::load(buf, &ctx)))
			})),
			Some(FileFormat::Stb | FileFormat::Stm) => Some(Self::with_load_fn(move || {
				let file = open(path)?;
				let buf = std::io::BufReader::new(file);
				
				Ok(Box::new(super::st::StFileView::load_not_stl(buf, &ctx)))
			})),
			Some(FileFormat::Stl) => Some(Self::with_load_fn(move || {
				let file = open(path)?;
				let buf = std::io::BufReader::new(file);
				
				Ok(Box::new(super::st::StFileView::load_stl(buf, &ctx)))
			})),
			Some(FileFormat::Anb) => Some(Self::with_load_fn(move || {
				let file = open(path)?;
				let buf = std::io::BufReader::new(file);
				
				Ok(Box::new(super::anb::AnbFileView::load(buf, &ctx)))
			})),
			Some(FileFormat::Image(ImageFormat::Png)) => Some(Self::with_load_fn(move || {
				let file = open(path)?;
				let buf = BufReader::new(file);
				
				Ok(Box::new(super::image::ImageFileView::load(buf, &ctx)))
			})),
			Some(FileFormat::Ltb) => Some(Self::with_load_fn(move || {
				let file = open(path)?;
				let buf = BufReader::new(file);
				
				Ok(Box::new(super::ltb::LtbFileView::load(buf, &ctx)))
			})),
			_ => None,
		}
	}
	
	fn with_load_fn(f: impl FnOnce() -> anyhow::Result<Box<dyn FileView>> + Send + 'static) -> Self {
		let join_handle = std::thread::spawn(f);
		Self { state: LoaderState::Loading(join_handle) }
	}
	
	pub fn ui(&mut self, ui: &mut egui::Ui) -> FileViewEffect {
		match std::mem::replace(&mut self.state, LoaderState::Placeholder) {
			LoaderState::Loading(handle) => {
				if handle.is_finished() {
					self.state = match handle.join().unwrap() {
						Ok(view) => LoaderState::Loaded(view),
						Err(e) => LoaderState::Failed(Arc::new(e)),
					};
				} else {
					self.state = LoaderState::Loading(handle); // put it back!
				}
			},
			other => { self.state = other; }, // put it back!
		}
		
		let effect = match &mut self.state {
			LoaderState::Loading(_) => {
				ui.spinner();
				FileViewEffect::default()
			},
			LoaderState::Loaded(view) => view.ui(ui),
			LoaderState::Failed(e) => {
				let text = egui::RichText::new(e.to_string())
					.color(ui.visuals().error_fg_color)
					.monospace();
				ui.label(text);
				FileViewEffect::default()
			},
			LoaderState::Placeholder => {
				ui.label("if this shows up shit's broken");
				FileViewEffect::default()
			},
		};
		
		ui.take_available_space();
		effect
	}
}

fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<impl BufRead + Seek> {
	let reader = std::fs::File::open(path)?;
	Ok(std::io::BufReader::new(reader))
}
