pub mod editable;
pub mod tree;

use std::sync::Arc;
use yoke::Yoke;
use zerocopy::{FromBytes, LE, U64};

pub type ArcBytes = Yoke<&'static [u8], Arc<Vec<u8>>>;

pub fn pointer_slice<'a>(bytes: &'a [u8], start_offset: u64, count: u32) -> anyhow::Result<&'a [U64<LE>]> {
	let (start_offset_u, count_u) = (start_offset as usize, count as usize);
	let sliced_bytes = bytes.get(start_offset_u..)
		.ok_or_else(|| anyhow::anyhow!("pointer list out of bounds"))?;
	
	let (pointers, _) = <[U64<LE>]>::ref_from_prefix_with_elems(sliced_bytes, count_u)
		.map_err(|e| e.map_src(<[_]>::to_vec))?;
	Ok(pointers)
}
