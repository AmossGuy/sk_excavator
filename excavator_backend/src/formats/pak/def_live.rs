use excavator_backend_macros::EditableData;
use bevy_ecs::component::Component;

#[derive(EditableData, Component)]
pub struct Header {
	pub file_count: u32,
}
