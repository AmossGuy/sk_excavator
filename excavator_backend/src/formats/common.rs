pub type ArcBytes = yoke::Yoke<&'static [u8], std::sync::Arc<Vec<u8>>>;

pub trait TreeFormat: Send + Sync + 'static {}

pub trait EditableData {
	fn struct_name(&self) -> &str;
	fn display<T: EditableDataRenderer>(&mut self, renderer: &mut T);
}

pub trait EditableDataRenderer {
	type Dropdown<'a>: DropdownRenderer;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(&mut Self::Dropdown<'_>));
	fn field_f32(&mut self, name: &str, value: &mut f32);
	fn field_u16(&mut self, name: &str, value: &mut u16);
	fn field_u32(&mut self, name: &str, value: &mut u32);
	fn field_u64(&mut self, name: &str, value: &mut u64);
}

pub trait DropdownRenderer {
	fn choice(&mut self, name: &str, selected: bool) -> bool;
}

pub trait FieldDispatch {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str);
}

impl FieldDispatch for f32 {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_f32(name, self);
	}
}

impl FieldDispatch for u16 {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_u16(name, self);
	}
}

impl FieldDispatch for u32 {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_u32(name, self);
	}
}

impl FieldDispatch for u64 {
	fn dispatch(&mut self, renderer: &mut impl EditableDataRenderer, name: &str) {
		renderer.field_u64(name, self);
	}
}
