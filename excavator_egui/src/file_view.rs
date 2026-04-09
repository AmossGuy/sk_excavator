mod anb;
mod image;
mod ltb;
mod st;

use std::path::PathBuf;
use std::sync::Arc;

use crate::ExcavatorMessage;
use crate::file_read::{BytesLoadResult, FileBytes, ItemInfo};
use crate::plugins::ItemLoaders;

use self::anb::AnbFileView;
use self::image::ImageFileView;
use self::ltb::LtbFileView;
use self::st::StFileView;

use excavator_backend::formats::FileFormat;

#[derive(Default)]
pub struct FileViewSwitcher {
	state: SwitcherState,
}

#[derive(Default)]
enum SwitcherState {
	#[default]
	NoticeBlank,
	NoticeMulti,
	NoticeUnknown { item: ItemInfo },
	NoticePak { item: ItemInfo },
	Loading {
		item: ItemInfo,
		when_ready: WhenReadyFunc,
	},
	LoadError {
		item: ItemInfo,
		message: String,
	},
	View {
		item: ItemInfo,
		view: Box<dyn ItemView>,
	},
}

type WhenReadyFunc = fn(ItemInfo, FileBytes, &egui::Context) -> SwitcherState;

trait LoadSwitch {
	fn when_ready(item: ItemInfo, bytes: FileBytes, ctx: &egui::Context) -> SwitcherState;
}

impl<T> LoadSwitch for T where T: ItemView + 'static {
	fn when_ready(item: ItemInfo, bytes: FileBytes, ctx: &egui::Context) -> SwitcherState {
		let view = Box::new(T::new(bytes, ctx));
		SwitcherState::View { item, view }
	}
}

impl SwitcherState {
	#[expect(dead_code)] // Maybe it'll come in handy later? IDK I really just want to shut up the warnings about the unused item fields
	fn get_item(&self) -> Option<&ItemInfo> {
		match self {
			Self::NoticeBlank | Self::NoticeMulti => None,
			Self::NoticeUnknown { item } => Some(item),
			Self::NoticePak { item } => Some(item),
			Self::Loading { item, .. } => Some(item),
			Self::LoadError { item, .. } => Some(item),
			Self::View { item, .. } => Some(item),
		}
	}
	
	fn start_load<T: LoadSwitch>(item: &ItemInfo, ctx: &egui::Context) -> Self {
		let item = item.clone();
		let when_ready: WhenReadyFunc = T::when_ready;
		
		let loaders = ctx.plugin_or_default::<ItemLoaders>();
		let loader = &mut loaders.lock().bytes_loader;
		
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
	pub fn switch(&mut self, selection: &Vec<ItemInfo>, ctx: &egui::Context) {
		match selection.len() {
			0 => self.state = SwitcherState::NoticeBlank,
			1 => self.switch_single(&selection[0], ctx),
			2.. => self.state = SwitcherState::NoticeMulti,
		};
	}
	
	fn switch_single(&mut self, item: &ItemInfo, ctx: &egui::Context) {
		self.state = match FileFormat::from_filename(item.filename()) {
			Some(FileFormat::Pak) => SwitcherState::NoticePak { item: item.clone() },
			Some(FileFormat::Stb | FileFormat::Stl | FileFormat::Stm) => SwitcherState::start_load::<StFileView>(&item, ctx),
			Some(FileFormat::Image(::image::ImageFormat::Png)) => SwitcherState::start_load::<ImageFileView>(&item, ctx),
			Some(FileFormat::Anb) => SwitcherState::start_load::<AnbFileView>(&item, ctx),
			Some(FileFormat::Ltb) => SwitcherState::start_load::<LtbFileView>(&item, ctx),
			_ => SwitcherState::NoticeUnknown { item: item.clone() },
		};
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
			SwitcherState::NoticeUnknown { .. } => { ui.label("Unknown or unimplemented file type."); },
			SwitcherState::NoticePak { .. } => { ui.label("Archive selected; please select one of the files inside the archive."); },
			SwitcherState::Loading { .. } => { ui.spinner(); },
			SwitcherState::LoadError { item, message } => {
				ui.label(format!(
					"Error loading item \"{}\":\n{}",
					item.display_name_lossy().unwrap_or_default(),
					message,
				));
			},
			SwitcherState::View { item, view } => {
				let message = ui.push_id(("item view", item), |ui| view.ui(ui)).inner;
				return message;
			},
		};
		None
	}
}
