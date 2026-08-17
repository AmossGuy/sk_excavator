use std::{any::Any, convert::TryFrom, sync::Arc};
use yoke::Yoke;

pub type ArcBytes = Yoke<&'static [u8], Arc<Vec<u8>>>;

// Add variants as needed
pub enum FieldRef<'a> {
	F32(&'a f32),
	U16(&'a u16),
	U32(&'a u32),
}

impl<'a> TryFrom<&'a dyn Any> for FieldRef<'a> {
	type Error = anyhow::Error;
	
	fn try_from(value: &'a dyn Any) -> anyhow::Result<Self> {
		 if let Some(v) = value.downcast_ref() {
			  Ok(Self::F32(v))
		 } else if let Some(v) = value.downcast_ref() {
			  Ok(Self::U16(v))
		 } else if let Some(v) = value.downcast_ref() {
			  Ok(Self::U32(v))
		 } else {
			  Err(anyhow::anyhow!("unimplemented field type"))
		 }
	}
}


pub trait EditableData {
	fn struct_name(&self) -> &str;
	
	fn field_count(&self) -> usize;
	fn field_name(&self, index: usize) -> &str;
	fn field_ref(&self, index: usize) -> FieldRef<'_>;
	
	fn variant_count(&self) -> usize {
		0
	}
	fn variant_name(&self, index: usize) -> &str {
		panic!("`EditableStruct::variant_name called on non-enum")
	}
	fn variant_current(&self) -> usize {
		panic!("`EditableStruct::variant_current called on non-enum")
	}
}
