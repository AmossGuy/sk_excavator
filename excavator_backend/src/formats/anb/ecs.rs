use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

#[derive(Component, Reflect)]
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
