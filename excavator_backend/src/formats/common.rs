use std::sync::Arc;
use yoke::Yoke;

pub type ArcBytes = Yoke<&'static [u8], Arc<Vec<u8>>>;

// Add variants as needed
pub enum FieldRef<'a> {
	F32(&'a f32),
	U16(&'a u16),
	U32(&'a u32),
}

macro_rules! field_ref_from {
	($type:ty, $variant:ident) => {
		impl<'a> From<&'a $type> for FieldRef<'a> {
			fn from(value: &'a $type) -> Self {
				Self::$variant(value)
			}
		}
	};
}

field_ref_from!(f32, F32);
field_ref_from!(u16, U16);
field_ref_from!(u32, U32);

pub trait EditableData {
	fn struct_name(&self) -> &str;
	
	fn field_count(&self) -> usize;
	fn field_name(&self, index: usize) -> &str;
	fn field_ref(&self, index: usize) -> FieldRef<'_>;
	
	fn variant_count(&self) -> usize {
		0
	}
	fn variant_name(&self, index: usize) -> &str {
		let _ = index;
		panic!("`EditableStruct::variant_name called on non-enum")
	}
	fn variant_current(&self) -> usize {
		panic!("`EditableStruct::variant_current called on non-enum")
	}
}
