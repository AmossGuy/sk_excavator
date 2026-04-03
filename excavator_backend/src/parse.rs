use bstr::BString;
use std::fmt;
use std::io::{BufRead, Seek, SeekFrom};
use std::marker::PhantomData;
use std::mem::size_of;
use zerocopy::FromBytes;

// TODO: make errors more specific
pub struct ParseError;
pub type ParseResult<T> = Result<T, ParseError>;

impl From<std::io::Error> for ParseError {
	fn from(_value: std::io::Error) -> Self {
		Self
	}
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		"file parsing error (todo: add more info)".fmt(f)
	}
}

pub struct ParseReader<R: BufRead + Seek> {
	reader: R,
}

impl<R: BufRead + Seek> ParseReader<R> {
	pub fn new(reader: R) -> Self {
		Self { reader }
	}
	
	pub fn cursor(&mut self, offset: u64) -> ParseResult<ParseCursor<'_, R>> {
		self.reader.seek(SeekFrom::Start(offset))?;
		Ok(ParseCursor { parse_reader: self })
	}
	
	pub fn read_struct<T: FromBytes>(&mut self, offset: u64) -> ParseResult<T> {
		self.cursor(offset)?.read_struct::<T>()
	}
	
	pub fn read_struct_array<T: FromBytes>(&mut self, offset: u64, entry_count: u64) -> ParseResult<ReadStructArray<'_, T, R>> {
		Ok(ReadStructArray::new(self.cursor(offset)?, entry_count))
	}

	pub fn read_null_terminated_string(&mut self, offset: u64) -> ParseResult<BString> {
		self.cursor(offset)?.read_null_terminated_string()
	}
}

pub struct ParseCursor<'a, R: BufRead + Seek> {
	parse_reader: &'a mut ParseReader<R>,
}

impl<'a, R: BufRead + Seek> ParseCursor<'a, R> {
	fn reader(&mut self) -> &mut R {
		&mut self.parse_reader.reader
	}
	
	pub fn read_struct<T: FromBytes>(&mut self) -> ParseResult<T> {
		let r#struct = T::read_from_io(self.reader())?;
		Ok(r#struct)
	}
	
	pub fn read_null_terminated_string(&mut self) -> ParseResult<BString> {
		let mut buf = Vec::new();
		self.reader().read_until(0, &mut buf)?;
		if buf.last() != Some(&0) { return Err(ParseError); } // Make sure we found the null terminator
		buf.pop(); // Remove the null terminator
		Ok(buf.into())
	}
	
	pub fn stream_position(&mut self) -> std::io::Result<u64> {
		self.reader().stream_position()
	}
}

pub struct ReadStructArray<'a, T: FromBytes, R: BufRead + Seek> {
	cursor: ParseCursor<'a, R>,
	remaining_count: u64,
	phantom: PhantomData<fn() -> T>,
}

impl<'a, T: FromBytes, R: BufRead + Seek> ReadStructArray<'a, T, R> {
	pub fn new(cursor: ParseCursor<'a, R>, remaining_count: u64) -> Self {
		Self { cursor, remaining_count, phantom: PhantomData }
	}
	
	// This is quibbling, but I don't want to artificially limit what you can pass in when usize is 32-bit
	pub fn nth_u64(&mut self, n: u64) -> Option<ParseResult<T>> {
		if self.remaining_count <= n { self.remaining_count = 0; return None; }
		self.remaining_count -= n;
		
		// This would just be `n * size_of::<T>()` if there weren't three different number types involved and I wasn't being careful with error handling.
		let Some(bytes_to_skip) = (|| {
			let (size_of_t, n) = (i64::try_from(size_of::<T>()).ok()?, i64::try_from(n).ok()?);
			size_of_t.checked_mul(n)
		})() else {
			// When I get to adding error info, EOF makes the most sense here I think
			return Some(Err(ParseError));
		};
		// That was six lines for one multiplication. Amazing. I refuse to resort to a conversion that isn't checked properly!
		
		let seek_result = self.cursor.parse_reader.reader.seek_relative(bytes_to_skip);
		if let Err(e) = seek_result { return Some(Err(e.into())); }
		
		self.next()
	}
}

impl<'a, T: FromBytes, R: BufRead + Seek> Iterator for ReadStructArray<'a, T, R> {
	type Item = ParseResult<T>;
	
	fn next(&mut self) -> Option<ParseResult<T>> {
		if self.remaining_count == 0 { return None; }
		self.remaining_count -= 1;
		
		let read_result = self.cursor.read_struct::<T>();
		if read_result.is_err() { self.remaining_count = 0; } // Stop the iterator if there's an error
		Some(read_result)
	}
	
	// This function's basically only here for completeness. It's a bit silly. Use `nth_u64`.
	fn nth(&mut self, n: usize) -> Option<ParseResult<T>> {
		// This conversion could only fail if usize was bigger than u64. For the foreseeable future, that will never come up in practice, but I have to write something for that case anyway.
		match n.try_into() {
			Ok(n_u64) => self.nth_u64(n_u64),
			Err(_) => { self.remaining_count = 0; None },
		}
	}
	
	fn size_hint(&self) -> (usize, Option<usize>) {
		match self.remaining_count {
			0 => (0, Some(0)),
			1.. => (1, self.remaining_count.try_into().ok())
		}
	}
}
