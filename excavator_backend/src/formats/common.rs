use bevy_reflect::{PartialReflect, Reflect};
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

impl<'a> TryFrom<&'a dyn Reflect> for FieldRef<'a> {
	type Error = anyhow::Error;
	
	fn try_from(value: &'a dyn Reflect) -> anyhow::Result<Self> {
		Self::try_from(value.as_any())
	}
}

impl<'a> TryFrom<&'a dyn PartialReflect> for FieldRef<'a> {
	type Error = anyhow::Error;
	
	fn try_from(value: &'a dyn PartialReflect) -> anyhow::Result<Self> {
		let full_reflect = value.try_as_reflect()
			.ok_or_else(|| anyhow::anyhow!("could not reflect"))?;
		Self::try_from(full_reflect)
	}
}
