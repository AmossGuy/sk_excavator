mod ecs;
mod load;
mod raw;
mod save;

pub use ecs::Header;
pub use load::load_from_bytes;
pub use save::save_from_world;

pub trait EditableStruct {
	fn struct_name(&self) -> &str;
	fn number_of_fields(&self) -> usize;
	fn field_name(&self, index: usize) -> Option<&str>;
	fn field_ref(&self, index: usize) -> Option<&dyn std::any::Any>;
	fn field_mut(&mut self, index: usize) -> Option<&mut dyn std::any::Any>;
}
