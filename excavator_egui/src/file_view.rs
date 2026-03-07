mod anb;
mod hex;
mod image;
mod st;

use std::path::PathBuf;
use std::sync::Arc;

use crate::ExcavatorMessage;
use crate::file_read::{BytesLoadResult, FileBytes, ItemInfo};
use crate::plugins::ItemLoaders;

use self::anb::AnbFileView;
use self::hex::hexedit_ui;
use self::image::ImageFileView;
use self::st::StFileView;
use excavator_formats::util_binary::{ParserStruct, ParserReflect};
use excavator_formats::anb::AnbHeader;

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
	HexView {
		item: ItemInfo,
		bytes: FileBytes,
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

struct HexLoadSwitch;

impl LoadSwitch for HexLoadSwitch {
	fn when_ready(item: ItemInfo, bytes: FileBytes, _ctx: &egui::Context) -> SwitcherState {
		SwitcherState::HexView { item, bytes }
	}
}

impl SwitcherState {
	fn get_item(&self) -> Option<&ItemInfo> {
		match self {
			Self::NoticeBlank | Self::NoticeMulti => None,
			Self::NoticeUnknown { item } => Some(item),
			Self::NoticePak { item } => Some(item),
			Self::Loading { item, .. } => Some(item),
			Self::LoadError { item, .. } => Some(item),
			Self::View { item, .. } => Some(item),
			Self::HexView { item, .. } => Some(item),
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
	pub fn switch(&mut self, selection: &Vec<ItemInfo>, is_hex_editor_on: bool, ctx: &egui::Context) {
		match selection.len() {
			0 => self.state = SwitcherState::NoticeBlank,
			1 => self.switch_single(&selection[0], is_hex_editor_on, ctx),
			2.. => self.state = SwitcherState::NoticeMulti,
		};
	}
	
	pub fn switch_same(&mut self, is_hex_editor_on: bool, ctx: &egui::Context) {
		if let Some(item) = self.state.get_item() {
			let item = item.clone();
			self.switch_single(&item, is_hex_editor_on, ctx);
		}
	}
	
	fn switch_single(&mut self, item: &ItemInfo, is_hex_editor_on: bool, ctx: &egui::Context) {
		if is_hex_editor_on {
			self.state = SwitcherState::start_load::<HexLoadSwitch>(&item, ctx);
			return;
		}
		
		let extension = item.extension();
		
		self.state = match extension {
			Some(b"pak") => SwitcherState::NoticePak { item: item.clone() },
			Some(b"stb" | b"stl" | b"stm") => SwitcherState::start_load::<StFileView>(&item, ctx),
			Some(b"png") => SwitcherState::start_load::<ImageFileView>(&item, ctx),
			Some(b"anb") => SwitcherState::start_load::<AnbFileView>(&item, ctx),
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
			SwitcherState::View { view, .. } => { return view.ui(ui); },
			SwitcherState::HexView { bytes, item } => {
				let parse = match item.extension() {
					Some(b"anb") => ParserStruct::<AnbHeader>::new(bytes.as_slice(), 0).retrieve().ok().map(|x| x as &dyn ParserReflect),
					_ => None,
				};
				hexedit_ui(bytes, parse, ui);
			},
		};
		None
	}
}
