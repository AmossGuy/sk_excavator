use excavator_backend::formats::common::FieldRef;
use bevy_reflect::{PartialReflect, enums::{DynamicEnum, Enum}, structs::{DynamicStruct, Struct}};

#[must_use = "value must be used to apply edits"]
pub fn struct_edit_ui<T: Struct>(ui: &mut egui::Ui, item: &T) -> Option<DynamicStruct> {
	let mut edited: Option<DynamicStruct> = None;
	
	for (name, value) in item.iter_fields() {
		ui.label(name);
		
		match FieldRef::try_from(value) {
			Ok(v) => {
				if let Some(new_value) = field_edit_widget(ui, v) {
					edited.get_or_insert_default().insert_boxed(name, new_value);
				}
			},
			Err(e) => {
				let warn_fg_color = ui.visuals().warn_fg_color;
				ui.colored_label(warn_fg_color, e.to_string());
			},
		}
		
		ui.end_row();
	}
	
	edited
}

#[must_use = "value must be used to apply edits"]
pub fn enum_edit_ui<T: Enum>(ui: &mut egui::Ui, item: &T) -> Option<DynamicEnum> {
	let mut edited: Option<DynamicEnum> = None;
	
	ui.label("enum variant");
	egui::ComboBox::from_id_salt("enum variant")
		.selected_text(item.variant_name())
		.show_ui(ui, |ui| {
			ui.label("todo");
		});
	ui.end_row();
	
	ui.label("todo");
	
	edited
}

#[must_use = "value must be used to apply edits"]
pub fn field_edit_widget(ui: &mut egui::Ui, value: FieldRef<'_>) -> Option<Box<dyn PartialReflect>> {
	macro_rules! drag_value {
		($ref:expr) => { {
			let mut value = ($ref).clone();
			let response = ui.add(egui::DragValue::new(&mut value));
			if response.changed() { Some(Box::new(value)) } else { None }
		} };
	}
	
	match value {
		FieldRef::F32(value) => drag_value!(value),
		FieldRef::U16(value) => drag_value!(value),
		FieldRef::U32(value) => drag_value!(value),
	}
}
