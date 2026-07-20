use super::{ecs, raw};
use bevy_ecs::{entity::Entity, system::Commands};
use zerocopy::FromBytes;

pub fn load_from_bytes(bytes: &[u8], commands: &mut Commands) -> Entity {
	load_header(bytes, commands)
}

fn load_header(bytes: &[u8], commands: &mut Commands) -> Entity {
	// early state of work on this... unwrapping okay for the moment
	let header_component = parse_header(bytes).unwrap();
	let header_ecs = commands.spawn(header_component);
	header_ecs.id()
}

fn parse_header(bytes: &[u8]) -> anyhow::Result<ecs::Header> {
	let (header_raw, _) = raw::Header::ref_from_prefix(bytes)
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"YCSN" {
		anyhow::bail!("wrong magic");
	}
	
	Ok(ecs::Header {
		unknown_04: header_raw.unknown_04.get(),
		unknown_08: header_raw.unknown_08.get(),
		unknown_0C: header_raw.unknown_0C.get(),
		unknown_10: header_raw.unknown_10.get(),
		unknown_14: header_raw.unknown_14.get(),
		unknown_18: header_raw.unknown_18.get(),
		unknown_1C: header_raw.unknown_1C.get(),
	})
}
