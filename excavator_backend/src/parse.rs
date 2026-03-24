use crate::io::FileBytes;
use bstr::BString;
use std::fmt;
use std::marker::PhantomData;
use zerocopy::FromBytes;

// TODO: make errors more specific
pub struct ParseError;
pub type ParseResult<T> = Result<T, ParseError>;

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		"file parsing error (todo: add more info)".fmt(f)
	}
}

pub struct ParseReader<L: ParseLogger = ()> {
	bytes: FileBytes,
	logger: L,
}

// right now we're loading in a whole file at once as handling it as a byte slice, but i'm writing these apis so they can work for reading the file on the fly as well, if we want to change it to that
impl<L: ParseLogger> ParseReader<L> {
	pub fn new(bytes: FileBytes, logger: L) -> Self {
		Self { bytes, logger }
	}
	
	pub fn read_struct<T: FromBytes>(&mut self, offset: impl TryInto<usize>) -> ParseResult<T> {
		self.read_struct_continued(offset).map(|(t, _)| t)
	}
	
	pub fn read_struct_continued<T: FromBytes>(&mut self, offset: impl TryInto<usize>) -> ParseResult<(T, ParseContinue<'_>)> {
		let offset = offset.try_into().map_err(|_| ParseError)?;
		
		let slice = self.bytes.get(offset..).ok_or(ParseError)?;
		match T::read_from_prefix(slice) {
			Ok((r#struct, _)) => {
				Ok((r#struct, ParseContinue::new(self.bytes.clone(), offset + std::mem::size_of::<T>())))
			},
			Err(_) => Err(ParseError),
		}
	}
	
	pub fn read_struct_array<T: FromBytes>(&mut self, offset: impl TryInto<usize>, entry_count: impl TryInto<usize>) -> ParseResult<impl Iterator<Item = ParseResult<T>>> {
		let offset = offset.try_into().map_err(|_| ParseError)?;
		let entry_count = entry_count.try_into().map_err(|_| ParseError)?;
		
		Ok(ReadStructArray {
			slice: self.bytes.get(offset..).ok_or(ParseError)?,
			remaining_count: entry_count,
			phantom: PhantomData,
		})
	}
	
	pub fn read_null_terminated_string(&mut self, offset: impl TryInto<usize>) -> ParseResult<BString> {
		let offset = offset.try_into().map_err(|_| ParseError)?;
		let slice = self.bytes.get(offset..).ok_or(ParseError)?;
		// TODO: This doesn't return an error when there's no null terminator
		Ok(slice.iter().take_while(|&&x| x != 0).copied().collect())
	}
}

pub struct ParseContinue<'a> {
	cropped_bytes: ParseResult<FileBytes>,
	// this api will make more sense when it's backed by a reader
	phantom: PhantomData<&'a [u8]>,
}

// Idea: Call this struct ParseCursor and make the current ParseReader methods sugar for creating a cursor and calling a corresponding method on it.
// For the .pak parsing I'm working on right this second, though, we only need one thing.
impl<'a> ParseContinue<'a> {
	fn new(bytes: FileBytes, offset: usize) -> Self {
		Self {
			cropped_bytes: bytes.cropped(offset..),
			phantom: PhantomData,
		}
	}
	
	pub fn archived_file(self, length: impl TryInto<usize>) -> ParseResult<FileBytes> {
		let length = length.try_into().map_err(|_| ParseError)?;
		self.cropped_bytes?.cropped(0..length)
	}
}

struct ReadStructArray<'a, T: FromBytes> {
	slice: &'a [u8],
	remaining_count: usize,
	phantom: PhantomData<fn() -> T>,
}

impl<'a, T: FromBytes> Iterator for ReadStructArray<'a, T> {
	type Item = ParseResult<T>;
	
	fn next(&mut self) -> Option<ParseResult<T>> {
		if self.remaining_count == 0 { return None; }
		
		match T::read_from_prefix(self.slice) {
			Ok((r#struct, remainder)) => { 
				self.slice = remainder;
				self.remaining_count -= 1;
				Some(Ok(r#struct))
			},
			Err(_) => Some(Err(ParseError)),
		}
	}
}

#[derive(Clone)]
pub struct ParseLogElement {
	offset: usize,
	length: usize,
	error_occurred: bool,
}

impl ParseLogElement {
	fn new(offset: usize, length: usize) -> Self {
		Self {
			offset, length,
			error_occurred: false,
		}
	}
}

pub trait ParseLogger {
	fn log(&mut self, element: ParseLogElement);
}

impl ParseLogger for () {
	fn log(&mut self, _element: ParseLogElement) {}
}
