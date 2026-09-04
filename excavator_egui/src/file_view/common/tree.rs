use crate::core::app::ExcavatorContext;
use crate::file_view::FileView;
use excavator_backend::formats::common::tree::TreeFormat;

use egui::Ui;

pub trait TreeFormatUi: TreeFormat {
	fn item_ui(&self, ui: &mut Ui, item: Self::AnyItemRef<'_>);
}

pub struct TreeFileView<T> {
	data: T,
}

impl<T> TreeFileView<T> where T: TreeFormat {
	pub fn new(data: T) -> Self {
		Self { data }
	}
}

impl<T> FileView for TreeFileView<T> where T: TreeFormatUi {
	fn ui(&mut self, ui: &mut Ui, _excavator: &ExcavatorContext) {
		egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
			let root = self.data.root_id();
			egui::containers::Frame::group(ui.style()).show(ui, |ui| {
				self.data.item_ui(ui, root);
			})
		});
	}
}
