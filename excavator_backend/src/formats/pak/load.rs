use crate::formats::common::ArcBytes;
use super::{def_live as live, def_raw as raw};

use bevy_ecs::{entity::Entity, /* hierarchy::ChildSpawner, */ world::World};
use zerocopy::FromBytes;

pub fn load_from_bytes(bytes: &ArcBytes, world: &mut World) -> anyhow::Result<Entity> {
	load_header(bytes, world)
}

fn load_header(bytes: &ArcBytes, world: &mut World) -> anyhow::Result<Entity> {
	let header_component = parse_header(bytes)?;
	let header_entity = world.spawn(header_component);
	
	Ok(header_entity.id())
}

fn parse_header(bytes: &ArcBytes) -> anyhow::Result<live::Header> {
	let (header_raw, _) = raw::Header::ref_from_prefix(bytes.get())
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"\0\0\0\0" {
		anyhow::bail!("wrong magic");
	}
	
	Ok(live::Header {
		file_count: header_raw.file_count.get(),
	})
}
