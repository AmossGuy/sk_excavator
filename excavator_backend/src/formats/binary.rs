//! General functions used for parsing binary formats.

use std::fmt::Debug;
use std::io::{Read, Seek};
use std::marker::PhantomData;

use binrw::{BinRead, BinResult, Endian, VecArgs};

use zerocopy::*;

pub(crate) fn read_pointers<R: Read + Seek>(reader: &mut R, count: usize) -> BinResult<Vec<u64>> {
	Vec::<u64>::read_options(
		reader,
		Endian::Little,
		VecArgs {
			count,
			inner: <_>::default(),
		},
	)
}

#[derive(Debug)]
pub enum ParserStructError {
	OutOfBounds,
	CastError(String),
}

impl std::fmt::Display for ParserStructError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		Debug::fmt(self, f) // another thing i'll get back to later
	}
}

impl std::error::Error for ParserStructError {}

pub struct ParserStruct<'a, T: ?Sized> {
	file: &'a [u8],
	offset: usize,
	phantom: PhantomData<&'a T>,
}

impl<'a, T: FromBytes + KnownLayout + Immutable + ?Sized> ParserStruct<'a, T> {
	pub fn new(file: &'a [u8], offset: usize) -> Self {
		Self { file, offset, phantom: PhantomData }
	}
	
	pub fn retrieve(&self) -> Result<&'a T, ParserStructError> {
		let slice = self.file.get(self.offset..)
			.ok_or(ParserStructError::OutOfBounds)?;
		let (retrieved, _) = T::ref_from_prefix(slice)
			.map_err(|e| ParserStructError::CastError(e.to_string()))?;
		Ok(retrieved)
	}
	
	pub fn retrieve_with_len(&self, length: usize) -> Result<&'a T, ParserStructError>
	where
		T: KnownLayout<PointerMetadata = usize>,
	{
		let slice = self.file.get(self.offset..)
			.ok_or(ParserStructError::OutOfBounds)?;
		let (retrieved, _) = T::ref_from_prefix_with_elems(slice, length)
			.map_err(|e| ParserStructError::CastError(e.to_string()))?;
		Ok(retrieved)
	}
	
	pub fn get_file(&self) -> &'a [u8] {
		self.file
	}
	
	pub fn get_offset(&self) -> usize {
		self.offset
	}
}

#[derive(Default, Copy, Clone)]
pub enum StructRole {
	#[default]
	Unspecified,
	CompressionBlock,
	CompressionLiterals,
}

pub trait ParserReflect: Debug {
	fn get_subordinates(&self, context: &mut ParserReflectContext);
	
	fn role(&self) -> StructRole {
		StructRole::default()
	}
}

pub struct ParserReflectContext<'a, 'b> {
	file: &'a [u8],
	consumer: &'b mut dyn FnMut(Result<&'a dyn ParserReflect, ParserStructError>),
	but_what_about_second_consumer: &'b mut dyn FnMut(Result<&'a [u8], ParserStructError>),
}

impl<'a, 'b> ParserReflectContext<'a, 'b> {
	pub fn new(file: &'a [u8], consumer: &'b mut dyn FnMut(Result<&'a dyn ParserReflect, ParserStructError>), but_what_about_second_consumer: &'b mut dyn FnMut(Result<&'a [u8], ParserStructError>)) -> Self {
		Self { file, consumer, but_what_about_second_consumer }
	}
	
	pub fn file(&self) -> &'a [u8] {
		self.file
	}
	
	pub fn bullshit(&mut self, thing: Result<&'a [u8], ParserStructError>) {
		(self.but_what_about_second_consumer)(thing);
	}
	
	pub fn ingest2<T: ParserReflect + 'a>(&mut self, thing: Result<&'a T, ParserStructError>) {
		self.ingest2_dyn(thing.map(|x| x as &'a dyn ParserReflect));
	}
	
	pub fn ingest2_dyn(&mut self, thing: Result<&'a dyn ParserReflect, ParserStructError>) {
		match thing {
			Ok(thing) => {
				(self.consumer)(Ok(thing));
				thing.get_subordinates(self);
			},
			Err(e) => {
				(self.consumer)(Err(e));
			},
		};
	}
	
	pub fn ingest<T: FromBytes + KnownLayout + Immutable + ParserReflect + 'a>(&mut self, pstruct: ParserStruct<'a, T>) {
		self.ingest2(pstruct.retrieve());
	}
	
	pub fn follow_pointer<T: FromBytes + KnownLayout + Immutable + ParserReflect + 'a>(&mut self, pointer: usize) {
		self.ingest(ParserStruct::<T>::new(self.file, pointer));
	}
}
