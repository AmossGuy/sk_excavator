use super::definition::*;
use hecs::{Entity, World};
use zerocopy::FromBytes;

pub fn load_from_bytes(bytes: &[u8], world: &mut World) -> Entity {
	load_header(bytes, world)
}

fn load_header(bytes: &[u8], world: &mut World) -> Entity {
	// early state of work on this... unwrapping okay for the moment
	let header_component = parse_header(bytes).unwrap();
	world.spawn((header_component,))
}

fn parse_header(bytes: &[u8]) -> anyhow::Result<Header> {
	let (header_raw, _) = HeaderRaw::ref_from_prefix(bytes)
		.map_err(|e| anyhow::anyhow!("{}", e))?;
	
	if header_raw.magic != *b"YCSN" {
		anyhow::bail!("wrong magic");
	}
	
	Ok(Header {
		unknown_04: header_raw.unknown_04.get(),
		unknown_08: header_raw.unknown_08.get(),
		unknown_0C: header_raw.unknown_0C.get(),
		unknown_10: header_raw.unknown_10.get(),
		unknown_14: header_raw.unknown_14.get(),
		unknown_18: header_raw.unknown_18.get(),
		unknown_1C: header_raw.unknown_1C.get(),
	})
}
