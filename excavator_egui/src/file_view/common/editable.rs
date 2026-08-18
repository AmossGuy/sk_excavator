use excavator_backend::formats::common::{DropdownRenderer, EditableData, EditableDataRenderer};

#[must_use = "value must be used to apply edits"]
pub fn edit_editable_data<T: EditableData + Clone>(ui: &mut egui::Ui, data: &T) -> Option<T> {
	let mut renderer = EguiDataRenderer::new(ui);
	let mut data_clone = data.clone();
	
	data_clone.display(&mut renderer);
	
	renderer.changed.then_some(data_clone)
}

struct EguiDataRenderer<'a> {
	ui: &'a mut egui::Ui,
	changed: bool,
}

impl<'a> EguiDataRenderer<'a> {
	fn new(ui: &'a mut egui::Ui) -> Self {
		Self { ui, changed: false }
	}
}

impl EditableDataRenderer for EguiDataRenderer<'_> {
	type Dropdown<'a> = EguiDropdownRenderer<'a>;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(&mut Self::Dropdown<'_>)) {
		let ui = &mut self.ui;
		ui.label(name);
		egui::ComboBox::from_id_salt(name)
			.selected_text(selected_text)
			.show_ui(ui, |ui| {
				let mut dd_renderer = EguiDropdownRenderer::new(ui);
				contents(&mut dd_renderer);
				if dd_renderer.changed {
					self.changed = true;
				}
			});
		ui.end_row();
	}
	
	fn field_f32(&mut self, name: &str, value: &mut f32) {
		let ui = &mut self.ui;
		ui.label(name);
		let response = ui.add(egui::DragValue::new(value));
		ui.end_row();
		
		if response.changed() {
			self.changed = true;
		}
	}
	
	fn field_u16(&mut self, name: &str, value: &mut u16) {
		let ui = &mut self.ui;
		ui.label(name);
		let response = ui.add(egui::DragValue::new(value));
		ui.end_row();
		
		if response.changed() {
			self.changed = true;
		}
	}
	
	fn field_u32(&mut self, name: &str, value: &mut u32) {
		let ui = &mut self.ui;
		ui.label(name);
		let response = ui.add(egui::DragValue::new(value));
		ui.end_row();
		
		if response.changed() {
			self.changed = true;
		}
	}
}

struct EguiDropdownRenderer<'a> {
	ui: &'a mut egui::Ui,
	changed: bool,
}

impl<'a> EguiDropdownRenderer<'a> {
	fn new(ui: &'a mut egui::Ui) -> Self {
		Self { ui, changed: false }
	}
}

impl DropdownRenderer for EguiDropdownRenderer<'_> {
	fn choice(&mut self, name: &str, selected: bool) -> bool {
		let ui = &mut self.ui;
		let clicked = ui.selectable_label(selected, name).clicked();
		if clicked && !selected {
			self.changed = true;
		}
		clicked
	}
}
