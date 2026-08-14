use excavator_backend::formats::common::{DropdownRenderer, EditableDataRenderer};

pub struct EguiDataRenderer<'a> {
	pub ui: &'a mut egui::Ui,
}

impl EditableDataRenderer for EguiDataRenderer<'_> {
	type Dropdown<'a> = EguiDropdownRenderer<'a>;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(Self::Dropdown<'_>)) {
		let ui = &mut self.ui;
		ui.label(name);
		egui::ComboBox::from_id_salt(name)
			.selected_text(selected_text)
			.show_ui(ui, |ui| {
				contents(EguiDropdownRenderer { ui })
			});
		ui.end_row();
	}
	
	fn field_f32(&mut self, name: &str, value: &mut f32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u16(&mut self, name: &str, value: &mut u16) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u32(&mut self, name: &str, value: &mut u32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
}

pub struct EguiDropdownRenderer<'a> {
	pub ui: &'a mut egui::Ui,
}

impl DropdownRenderer for EguiDropdownRenderer<'_> {
	fn choice(&mut self, name: &str, selected: bool) -> bool {
		let ui = &mut self.ui;
		ui.selectable_label(selected, name).clicked()
	}
}
