use super::EditableStruct;

#[expect(non_snake_case)]
pub struct Header {
	pub unknown_04: u32,
	pub unknown_08: u32,
	pub unknown_0C: u32,
	pub unknown_10: u32,
	pub unknown_14: u32,
	pub unknown_18: u32,
	pub unknown_1C: u32,
}

// this'll be cool once i bring in my cool new proc macro skills (soon)
impl EditableStruct for Header {
	fn struct_name(&self) -> &str {
		"Header"
	}
	
	fn number_of_fields(&self) -> usize {
		7
	}
	
	fn field_name(&self, index: usize) -> Option<&str> {
		match index {
			0 => Some("unknown_04"),
			1 => Some("unknown_08"),
			2 => Some("unknown_0C"),
			3 => Some("unknown_10"),
			4 => Some("unknown_14"),
			5 => Some("unknown_18"),
			6 => Some("unknown_1C"),
			_ => None,
		}
	}
	
	fn field_ref(&self, index: usize) -> Option<&dyn std::any::Any> {
		match index {
			0 => Some(&self.unknown_04),
			1 => Some(&self.unknown_08),
			2 => Some(&self.unknown_0C),
			3 => Some(&self.unknown_10),
			4 => Some(&self.unknown_14),
			5 => Some(&self.unknown_18),
			6 => Some(&self.unknown_1C),
			_ => None,
		}
	}
		
	fn field_mut(&mut self, index: usize) -> Option<&mut dyn std::any::Any> {
		match index {
			0 => Some(&mut self.unknown_04),
			1 => Some(&mut self.unknown_08),
			2 => Some(&mut self.unknown_0C),
			3 => Some(&mut self.unknown_10),
			4 => Some(&mut self.unknown_14),
			5 => Some(&mut self.unknown_18),
			6 => Some(&mut self.unknown_1C),
			_ => None,
		}
	}
}
