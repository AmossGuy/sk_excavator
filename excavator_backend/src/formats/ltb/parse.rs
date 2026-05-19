use crate::parse::*;
use crate::formats::anb::decompress_wflz;
use super::*;

use std::io::{BufRead, Seek};
use zerocopy::{LittleEndian as LE, U16, U32, U64};
use anyhow::Context;

pub fn parse_ltb<R: BufRead + Seek>(reader: &mut R) -> anyhow::Result<ParsedLtb> {
	let mut reader = ParseReader::new(reader);
	let header = reader.read_struct::<LtbHeader>(0)?;
	
	let image_pointers = read_struct_array_from_row::<U64<LE>>(&mut reader, &header.rows[7]).context("image pointers")?;
	let meta = read_struct_array_from_row::<ImageMetadata>(&mut reader, &header.rows[2]).context("image metadata")?;
	
	let images = std::iter::zip(image_pointers, meta).map(|(poin, meta)| {
		let mut cursor = reader.cursor(poin.get())?;
		let mut data = vec![0; meta.data_size.get() as usize].into_boxed_slice();
		cursor.inner_reader().read_exact(&mut data)?;
		Ok(ParsedImage { data, meta })
	}).collect::<Vec<_>>();
	
	let (layer_metadata, layer_metadata_weird_number) = read_struct_array_from_row_2::<LayerMetadata>(&mut reader, &header.rows[0], true).context("layer metadata")?;
	let chunkmap_data = read_struct_array_from_row::<U32<LE>>(&mut reader, &header.rows[3]).context("chunkmap data")?;
	let chunkmap_data = chunkmap_data.iter().map(|x| x.get()).collect::<Vec<_>>();
	let tilemap_data = read_struct_array_from_row::<U16<LE>>(&mut reader, &header.rows[4]).context("tilemap data")?;
	let tilemap_data = tilemap_data.iter().map(|x| x.get()).collect::<Vec<_>>();
	
	Ok(ParsedLtb { header, images, layer_metadata, chunkmap_data, tilemap_data })
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
	
	pub fn dump_everything(&self, folder_path: std::path::PathBuf) -> anyhow::Result<()> {
		use std::fs::{create_dir_all, File};
		use std::io::Write;
		
		create_dir_all(&folder_path)?;
		
		for (i, image_result) in self.images.iter().enumerate() {
			let image = image_result.as_ref().map_err(|e| anyhow::anyhow!("error loading image #{}: {}", i, e))?;
			
			let mut image_write_file = File::create(folder_path.join(format!("image {}.wflz", i)))?;
			image_write_file.write_all(&image.data)?;
		}
		
		Ok(())
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
		self.meta.palettes[0] != u32::MAX
	}
	
	pub fn meta_debug(&self) -> String {
		format!("{:?}", self.meta)
	}
}
