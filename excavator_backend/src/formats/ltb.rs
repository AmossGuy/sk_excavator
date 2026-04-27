use crate::parse::*;
use crate::formats::anb::decompress_wflz;
use std::io::{BufRead, Seek};
use zerocopy::{FromBytes, LittleEndian as LE, U16, U32, U64};
use zerocopy_derive::*;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_SIZE.pow(2);

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
#[allow(non_snake_case)]
struct LtbHeader {
	unknown_00: U32<LE>,
	unknown_04: U32<LE>,
	unknown_08: U32<LE>,
	unknown_0C: U32<LE>,
	rows: [LtbHeaderRow; 8],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct LtbHeaderRow {
	unknown: U32<LE>, // usually the same as entry_count below. what does that mean?
	entry_count: U32<LE>,
	entry_pointer: U64<LE>,
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct LayerMetadata {
	name: [u8; 32],
	unknown_arr1: [U32<LE>; 14],
	unknown_38: U32<LE>,
	chunkmap_width: U32<LE>,
	chunkmap_height: U32<LE>,
	chunkmap_offset: U32<LE>,
	unknown_arr2: [U32<LE>; 6],
}

#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct ImageMetadata {
	unknown_00: U32<LE>,
	is_compressed: U32<LE>,
	image_width: U32<LE>,
	image_height: U32<LE>,
	unknown_more: [U32<LE>; 14],
	data_size: U32<LE>,
}

pub fn parse_ltb<R: BufRead + Seek>(reader: &mut R) -> anyhow::Result<ParsedLtb> {
	let mut reader = ParseReader::new(reader);
	let header = reader.read_struct::<LtbHeader>(0)?;
	
	let image_pointers = read_struct_array_from_row::<U64<LE>>(&mut reader, &header.rows[7])?;
	let meta = read_struct_array_from_row::<ImageMetadata>(&mut reader, &header.rows[2])?;
	
	let images = std::iter::zip(image_pointers, meta).map(|(poin, meta)| {
		let mut cursor = reader.cursor(poin.get())?;
		let mut data = vec![0; meta.data_size.get() as usize].into_boxed_slice();
		cursor.inner_reader().read_exact(&mut data)?;
		Ok(ParsedImage { data, meta })
	}).collect::<Vec<_>>();
	
	let layer_metadata = read_struct_array_from_row::<LayerMetadata>(&mut reader, &header.rows[0])?;
	let chunkmap_data = read_struct_array_from_row::<U32<LE>>(&mut reader, &header.rows[3])?;
	let chunkmap_data = chunkmap_data.iter().map(|x| x.get()).collect::<Vec<_>>();
	let tilemap_data = read_struct_array_from_row::<U16<LE>>(&mut reader, &header.rows[4])?;
	let tilemap_data = tilemap_data.iter().map(|x| x.get()).collect::<Vec<_>>();
	
	Ok(ParsedLtb { header, images, layer_metadata, chunkmap_data, tilemap_data })
}

fn read_struct_array_from_row<T: FromBytes>(reader: &mut ParseReader<impl BufRead + Seek>, row: &LtbHeaderRow) -> anyhow::Result<Vec<T>> {
	let array_count = row.entry_count.get();
	let array_pointer = row.entry_pointer.get();
	
	let vec = reader
		.read_struct_array::<T>(array_pointer, array_count.into())?
		.collect::<Result<Vec<_>, _>>()?;
	Ok(vec)
}

pub struct ParsedLtb {
	header: LtbHeader,
	images: Vec<anyhow::Result<ParsedImage>>,
	layer_metadata: Vec<LayerMetadata>,
	chunkmap_data: Vec<u32>,
	tilemap_data: Vec<u16>,
}

impl ParsedLtb {
	pub fn test(&self) -> String {
		format!("{:?}", self.header)
	}
	
	pub fn images(&self) -> impl Iterator<Item = &anyhow::Result<ParsedImage>> {
		self.images.iter()
	}
	
	pub fn debug_layers(&self) -> impl Iterator<Item = (usize, String)> {
		self.layer_metadata.iter().enumerate().map(|(i, layer)| {
			let name = String::from_utf8_lossy(layer.name.split(|&x| x == 0).nth(0).unwrap_or_default());
			(i, format!("(name: {}, width in chunks: {}, height in chunks: {})", name, layer.chunkmap_width, layer.chunkmap_height))
		})
	}
	
	pub fn layer_count(&self) -> usize {
		self.layer_metadata.len()
	}
	
	pub fn chunk_grid_size(&self, layer: usize) -> [u32; 2] {
		let metadata = &self.layer_metadata[layer];
		[metadata.chunkmap_width.get(), metadata.chunkmap_height.get()]
	}
	
	pub fn iterate_chunk_offsets(&self, layer: usize) -> impl Iterator<Item = u32> {
		let metadata = &self.layer_metadata[layer];
		let start = metadata.chunkmap_offset.get() as usize;
		let length = metadata.chunkmap_width.get() as usize * metadata.chunkmap_height.get() as usize;
		self.chunkmap_data[start..start+length].iter().copied()
	}
	
	pub fn get_chunk_data(&self, offset: usize) -> &[u16] {
		&self.tilemap_data[offset..offset+CHUNK_AREA]
	}
}

pub struct ParsedImage {
	data: Box<[u8]>,
	meta: ImageMetadata,
}

impl ParsedImage {
	pub fn decompress(&self) -> anyhow::Result<Box<[u8]>> {
		match self.meta.is_compressed.get() {
			0 => Ok(self.data.clone()),
			1 => decompress_wflz(&mut std::io::Cursor::new(&self.data)).map_err(|e| e.into()),
			x => Err(anyhow::anyhow!("Invalid value for compression toggle: {}", x)),
		}
	}
	
	pub fn size(&self) -> [u32; 2] {
		[self.meta.image_width.get(), self.meta.image_height.get()]
	}
	
	pub fn paletted(&self) -> bool {
		// Is this really how this is determined?
		self.meta.unknown_more[2] != u32::MAX
	}
	
	pub fn meta_debug(&self) -> String {
		let meta = &self.meta;
		// let more_count = meta.unknown_more.iter().filter(|x| x.get() != u32::MAX).count();
		format!(
			"unknown a: {}, compressed: {}, width: {}, height: {}, unknown 1: 0x{:X}, unknown 2: {}, unknown 3: 0x{:X}, unknown last: {}",
			meta.unknown_00, meta.is_compressed, meta.image_width, meta.image_height,
			meta.unknown_more[0], meta.unknown_more[1], meta.unknown_more[2], meta.unknown_more[13],
		)
	}
}
