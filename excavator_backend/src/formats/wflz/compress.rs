// wow all this is terrible code. i hate this rn

use super::{WflzBlock, WflzHeader, WFLZ_MAGIC};
use zerocopy::{FromBytes, FromZeros, IntoBytes, LE, U16, U32};

fn wflz_hash(four_bytes: &[u8; 4]) -> u16 {
	let x = U32::<LE>::from_bytes(*four_bytes).get();
	(x.wrapping_mul(2654435761) >> 16) as u16
}

fn wflz_compare(slice_a: &[u8], slice_b: &[u8]) -> usize {
	std::iter::zip(slice_a, slice_b).take_while(|(a, b)| a == b).count()
}

const WFLZ_MIN_MATCH_LEN: usize = std::mem::size_of::<WflzBlock>() + 1;
const WFLZ_MAX_MATCH_DIST: usize = 0xFFFF;
const WFLZ_MAX_LITERALS: usize = 0xFF;

struct BlockBuild {
	backref_dist: u16,
	backref_length: u8,
	
	literal_start: usize,
	literal_length: u8,
}

impl BlockBuild {
	fn from_read_index(read_index: usize) -> Self {
		Self {
			backref_dist: 0,
			backref_length: 0,
			literal_start: read_index,
			literal_length: 0,
		}
	}
	
	fn write(&self, input: &[u8], output: &mut Vec<u8>) {
		output.extend(WflzBlock {
			backref_dist: U16::new(self.backref_dist),
			backref_length: self.backref_length,
			literals_length: self.literal_length,
		}.as_bytes());
		output.extend(&input[self.literal_start..self.literal_start + usize::from(self.literal_length)]);
	}
}

pub fn compress(data: &[u8]) -> Vec<u8> {
	// hash table, with a starring role in this compression scheme
	// sequences of 4 bytes are turned into a 32-bit integer and hashed with wflz_hash; that hash is used as an index into this table
	// the values of this table are indexes into the data, pointing to where those bytes sequences can be found
	let mut wflz_dict = <[usize; 0x10000]>::new_box_zeroed().unwrap();
	
	// mut for shrinking our slice of the data as we read it (not modifying the data itself)
	// let mut cropped_data = data;
	
	// make space to write the header later
	// the first block being counted as part of the header is kind of awkward here, isn't it?
	let mut output = vec![0; std::mem::size_of::<WflzHeader>() - std::mem::size_of::<WflzBlock>()];
	
	let mut read_index: usize = 0;
	let mut block_build = BlockBuild::from_read_index(read_index);
	
	// firstly: the first MIN_MATCH_LEN bytes of the data are always written as literals
	for _ in 0..WFLZ_MIN_MATCH_LEN {
		let Some(cropped_data) = data.get(read_index..) else { break };
		
		if let Ok((four_bytes, _)) = <[u8; 4]>::ref_from_prefix(cropped_data) {
			let whash = wflz_hash(four_bytes);
			wflz_dict[usize::from(whash)] = read_index;
		}
		
		read_index += 1;
		block_build.literal_length += 1;
	}
	
	// main loop
	while data.len() - read_index >= WFLZ_MIN_MATCH_LEN {
		let cropped_data = &data[read_index..];
		
		let (four_bytes, _) = <[u8; 4]>::ref_from_prefix(cropped_data).unwrap();
		let whash = wflz_hash(four_bytes);
		let match_pos = wflz_dict[usize::from(whash)];
		let mut match_length = 0;
		
		let window_start = read_index.saturating_sub(WFLZ_MAX_MATCH_DIST);
		
		// "a match was found, ensure it really is a match and not a hash collision, and determine its length"
		// not quite like the original since zero fails as a sentinel with indexes instead of addresses...
		if match_pos >= window_start {
			match_length = wflz_compare(&data[read_index..], &data[match_pos..]);
		}
		
		if match_length >= WFLZ_MIN_MATCH_LEN {
			let match_dist = read_index - match_pos;
			
			block_build.write(data, &mut output);
			block_build = BlockBuild::from_read_index(read_index);
			read_index += match_length;
			
			block_build.backref_dist = match_dist.try_into().unwrap();
			block_build.backref_length = (match_length - WFLZ_MIN_MATCH_LEN + 1).try_into().unwrap();
		} else {
			// "output a literal byte: no entries for this position found, entry is too far away, entry was a hash collision, or the entry did not meet the minimum match length"
			
			// "if we've hit the max number of sequential literals, we need to output a compression block header"
			if usize::from(block_build.literal_length) == WFLZ_MAX_LITERALS {
				block_build.write(data, &mut output);
				block_build = BlockBuild::from_read_index(read_index);
			}
			
			block_build.literal_length += 1;
			read_index += 1;
		}
	}
	
	// final literals
	while read_index < data.len() {
		if usize::from(block_build.literal_length) == WFLZ_MAX_LITERALS {
			block_build.write(data, &mut output);
			block_build = BlockBuild::from_read_index(read_index);
		}
		
		block_build.literal_length += 1;
		read_index += 1;
	}
	block_build.write(data, &mut output); // no need for a new block this time since we're already done
	
	// add terminator block
	output.extend([0; 4]);
	
	// write header to the space we reserved at the beginning
	let output_len = output.len();
	let (header, _) = WflzHeader::mut_from_prefix(&mut output).unwrap();
	header.magic = WFLZ_MAGIC;
	header.compressed_size = U32::new((output_len - std::mem::size_of::<WflzHeader>()) as u32);
	header.decompressed_size = U32::new(data.len() as u32);
	
	return output;
}
