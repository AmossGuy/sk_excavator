use crate::ExcavatorMessage;
use crate::file_read::FileBytes;
use super::ItemView;

// use excavator_formats::anb::AnbHeader;
// use excavator_formats::util_binary::ParserStruct;

pub struct AnbFileView {
	#[expect(dead_code)] // todo
	bytes: FileBytes,
}

impl ItemView for AnbFileView {
	fn new(bytes: FileBytes, _ctx: &egui::Context) -> Self where Self: Sized {
		Self { bytes }
	}
	
	fn ui(&mut self, ui: &mut egui::Ui) -> Option<ExcavatorMessage> {
		ui.label("todo");
		/*
		let bytes = self.bytes.as_slice();
		let header = ParserStruct::<AnbHeader>::new(bytes, 0);
		let thing = header.retrieve().unwrap().get_subordinate_data(bytes).unwrap();
		ui.label(format!(
			"{:?}\n\n{:?}",
			header.retrieve().unwrap(),
			thing.retrieve().unwrap(),
		));
		*/
		None
	}
}
