// primarily based on https://www.burtleburtle.net/bob/c/lookup3.c

use std::num::Wrapping;
use zerocopy::{ByteOrder, FromBytes, FromZeros, IntoBytes, U32, LE};

// So a bunch of methods for Wrapping, including rotate_left, have been sitting around since 2018 without being stabilized. Annoying.
// https://github.com/rust-lang/rust/issues/32463
fn rotate_left(this: Wrapping<u32>, n: u32) -> Wrapping<u32> {
	Wrapping(this.0.rotate_left(n))
}

fn mix(a: &mut Wrapping<u32>, b: &mut Wrapping<u32>, c: &mut Wrapping<u32>) {
	*a -= *c; *a ^= rotate_left(*c,  4); *c += *b;
	*b -= *a; *b ^= rotate_left(*a,  6); *a += *c;
	*c -= *b; *c ^= rotate_left(*b,  8); *b += *a;
	*a -= *c; *a ^= rotate_left(*c, 16); *c += *b;
	*b -= *a; *b ^= rotate_left(*a, 19); *a += *c;
	*c -= *b; *c ^= rotate_left(*b,  4); *b += *a;
}

fn final_mix(a: &mut Wrapping<u32>, b: &mut Wrapping<u32>, c: &mut Wrapping<u32>) {
	*c ^= *b; *c -= rotate_left(*b, 14);
	*a ^= *c; *a -= rotate_left(*c, 11);
	*b ^= *a; *b -= rotate_left(*a, 25);
	*c ^= *b; *c -= rotate_left(*b, 16);
	*a ^= *c; *a -= rotate_left(*c,  4);
	*b ^= *a; *b -= rotate_left(*a, 14);
	*c ^= *b; *c -= rotate_left(*b, 24);
}

pub fn hash<E: ByteOrder>(data: &[u8], init_val: u32) -> u32 {
	// Modifying our reference to the data, not the data itself
	// I could put mut in the signature but I need somewhere to put this comment
	let mut data = data;
	
	let mut a = Wrapping(0xDEADBEEF) + Wrapping(data.len() as u32) + Wrapping(init_val);
	let mut b = a; let mut c = a;
	
	while let Ok((block, remainder)) = <[U32<E>; 3]>::ref_from_prefix(data) {
		a += block[0].get();
		b += block[1].get();
		c += block[2].get();
		mix(&mut a, &mut b, &mut c);
		
		data = remainder;
	}
	
	if data.len() == 0 {
		return c.0;
	}
	
	let mut final_block = <[U32<E>; 3]>::new_zeroed();
	final_block.as_mut_bytes()[0..data.len()].copy_from_slice(data);
	
	a += final_block[0].get();
	b += final_block[1].get();
	c += final_block[2].get();
	final_mix(&mut a, &mut b, &mut c);
	
	return c.0;
}

pub const INIT_VAL_SK: u32 = 123456789;

pub fn hash_sk(data: &[u8]) -> u32 {
	hash::<LE>(data, INIT_VAL_SK)
}
