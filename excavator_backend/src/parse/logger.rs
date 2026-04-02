use rust_lapper::{Interval, Lapper};
use std::io::Cursor;

#[derive(Clone)]
pub struct ParseLogElement {
	offset: u64,
	length: u64,
	error_occurred: bool,
}

impl ParseLogElement {
	fn new(offset: u64, length: u64) -> Self {
		Self {
			offset, length,
			error_occurred: false,
		}
	}
}

pub trait ParseLogger<R> {
	type Cur;
	type Out;
	fn cursor(&mut self, offset: u64) -> Self::Cur;
	fn segment(&mut self, reader: &mut R, cur: &mut Self::Cur);
	fn collect(self) -> Self::Out;
}

impl<R> ParseLogger<R> for () {
	type Cur = ();
	type Out = ();
	fn cursor(&mut self, _offset: u64) {}
	fn segment(&mut self, _reader: &mut R, _cur: &mut Self::Cur) {}
	fn collect(self) {}
}

trait Position {
	fn position(&self) -> u64;
}

// Not very general, but it's entirely fine since Cursor is the only reader we need this to work for
impl<T> Position for Cursor<T> {
	fn position(&self) -> u64 {
		Cursor::position(self)
	}
}

pub struct FullParseLogger {
	intervals: Vec<Interval<u64, ()>>,
}

impl FullParseLogger {
	pub fn new() -> Self {
		Self { intervals: Vec::new() }
	}
}

impl<R: Position> ParseLogger<R> for FullParseLogger {
	type Cur = FullParseLoggerCursor;
	type Out = Lapper<u64, ()>;
	
	fn cursor(&mut self, offset: u64) -> Self::Cur {
		FullParseLoggerCursor { prev_offset: offset }
	}
	
	fn segment(&mut self, reader: &mut R, cur: &mut Self::Cur) {
		let prev_offset = cur.prev_offset;
		let new_offset = reader.position();
		
		self.intervals.push(Interval { start: prev_offset, stop: new_offset, val: () });
		
		cur.prev_offset = new_offset;
	}
	
	fn collect(self) -> Lapper<u64, ()> {
		Lapper::new(self.intervals)
	}
}

pub struct FullParseLoggerCursor {
	prev_offset: u64,
}
