mod image;
mod st;

use std::path::PathBuf;
use std::sync::Arc;

use crate::ExcavatorMessage;
use crate::file_read::{BytesLoadResult, FileBytes, ItemInfo, ItemLoader};
use self::image::ImageFileView;
use self::st::StFileView;

#[derive(Default)]
pub struct FileViewSwitcher {
	state: SwitcherState,
}

#[derive(Default)]
enum SwitcherState {
	#[default]
	NoticeBlank,
	NoticeMulti,
	NoticeUnknown,
	NoticePak,
	Loading {
		item: ItemInfo,
		when_ready: WhenReadyFunc,
	},
	LoadError {
		item: ItemInfo,
		message: String
	},
	View {
		item: ItemInfo,
		view: Box<dyn ItemView>,
	},
}

type WhenReadyFunc = fn(ItemInfo, FileBytes, &egui::Context) -> SwitcherState;

type BytesLoader = ItemLoader<BytesLoadResult>;

impl SwitcherState {
	fn start_load<T: ItemView + 'static>(item: &ItemInfo, loader: &mut BytesLoader, ctx: &egui::Context) -> Self {
		let item = item.clone();
		let when_ready: WhenReadyFunc = |item, bytes, ctx| {
			let view = Box::new(T::new(bytes, ctx));
			Self::View { item, view }
		};
		
		if let Some(result) = loader.get_or_request(&item, ctx) {
			Self::make_loaded_state(result, &item, &when_ready, ctx)
		} else {
			Self::Loading { item, when_ready }
		}
	}
	
	fn make_loaded_state(load_result: Arc<BytesLoadResult>, item: &ItemInfo, when_ready: &WhenReadyFunc, ctx: &egui::Context) -> Self {
		let item = item.clone();
		match crate::file_read::slice_item(load_result, &item) {
			Ok(bytes) => when_ready(item, bytes, ctx),
			Err(message) => Self::LoadError { item, message },
		}
	}
}

trait ItemView {
	fn new(bytes: FileBytes, ctx: &egui::Context) -> Self where Self: Sized;
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage>;
}

impl FileViewSwitcher {
	pub fn switch(&mut self, selection: &Vec<ItemInfo>, loader: &mut BytesLoader, ctx: &egui::Context) {
		match selection.len() {
			0 => self.state = SwitcherState::NoticeBlank,
			1 => self.switch_single(&selection[0], loader, ctx),
			2.. => self.state = SwitcherState::NoticeMulti,
		};
	}
	
	fn switch_single(&mut self, item: &ItemInfo, loader: &mut BytesLoader, ctx: &egui::Context) {
		let extension = item.extension();
		
		self.state = match extension {
			Some(b"pak") => SwitcherState::NoticePak,
			// Some(b"stb" | b"stl" | b"stm") => SwitcherState::start_load::<StFileView>(item, loader, ctx),
			Some(b"png") => SwitcherState::start_load::<ImageFileView>(&item, loader, ctx),
			_ => SwitcherState::NoticeUnknown,
		};
		
		/*
		let view = match selection.extension() {
			Some(b"pak") => SingleView::Pak,
			Some(b"stb" | b"stl" | b"stm") => SingleView::St(StFileView::default()),
			Some(b"png") => SingleView::ImageLoading,
			_ => SingleView::Unknown,
		};
		self.state = SwitcherState::Single { item: selection.clone(), view };
		*/
	}
	
	pub fn update_from_load(&mut self, load_path: &PathBuf, load_result: Arc<BytesLoadResult>, ctx: &egui::Context) {
		match &self.state {
			SwitcherState::Loading { item, when_ready } if item.outer_path() == load_path => {
				self.state = SwitcherState::make_loaded_state(load_result, &item, &when_ready, ctx);
			},
			_ => {},
		};
	}
	
	pub fn add_view(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		match &mut self.state {
			SwitcherState::NoticeBlank => { ui.label("No files are selected."); },
			SwitcherState::NoticeMulti => { ui.label("Multiple files are selected."); },
			SwitcherState::NoticeUnknown => { ui.label("Unknown or unimplemented file type."); },
			SwitcherState::NoticePak => { ui.label("Archive selected; please select one of the files inside the archive."); },
			SwitcherState::Loading { .. } => { ui.spinner(); },
			SwitcherState::LoadError { item, message } => {
				ui.label(format!(
					"Error loading item \"{}\":\n{}",
					item.file_name_lossy().unwrap_or_default(),
					message,
				));
			},
			SwitcherState::View { item: _, view } => { return view.ui(ui); },
		};
		None
	}
}
