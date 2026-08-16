use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

#[derive(Component, Reflect)]
pub struct Header {
	pub file_count: u32,
}
