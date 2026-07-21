pub mod anb;
pub mod ltb;
pub mod pak;
pub mod st;
mod wflz;

use image::ImageFormat;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FileFormat {
	// archive
	Pak,
	
	// tabular data
	Stb,
	Stl,
	Stm,
	
	// graphics
	Anb,
	Image(ImageFormat),
	
	// level data
	Ltb,
	Lvb,
}

impl FileFormat {
	pub fn from_filename<T: AsRef<[u8]>>(path: T) -> Option<Self> {
		let filename = path.as_ref().split(|b| *b == b'/' || *b == b'\\').last().unwrap_or_default();
		let extension = filename.split(|b| *b == b'.').last().unwrap_or_default();
		Self::from_extension(extension)
	}
	
	// Could be pub, but I can't think of a reason to use it outside of from_filename
	fn from_extension<T: AsRef<[u8]>>(ext: T) -> Option<Self> {
		let ext = ext.as_ref();
		match ext.to_ascii_lowercase().as_slice() {
			b"pak" => Some(Self::Pak),
			b"stb" => Some(Self::Stb),
			b"stl" => Some(Self::Stl),
			b"stm" => Some(Self::Stm),
			b"anb" => Some(Self::Anb),
			b"ltb" => Some(Self::Ltb),
			b"lvb" => Some(Self::Lvb),
			
			// If it isn't one of the extensions above, see whether it's one of the extensions the image crate knows.
			// We only return None if ImageFormat::from_extension does.
			_ => str::from_utf8(ext).ok()
				.and_then(|ext| ImageFormat::from_extension(ext))
				.map(|format| Self::Image(format)),
		}
	}
}

trait RawField {
	type Parsed;
	fn parse(&self) -> Self::Parsed;
	fn unparse(parsed: Self::Parsed) -> Self;
}

macro_rules! raw_field_self {
	($ty:ty) => {
		impl RawField for $ty {
			type Parsed = Self;
			fn parse(&self) -> Self { self.clone() }
			fn unparse(parsed: Self) -> Self { parsed.clone() }
		}
	};
}

macro_rules! raw_field_zerocopy {
	($zerocopy_ty:ty, $parsed_ty:ty) => {
		impl<O: zerocopy::ByteOrder> RawField for $zerocopy_ty {
			type Parsed = $parsed_ty;
			fn parse(&self) -> $parsed_ty { self.get() }
			fn unparse(parsed: $parsed_ty) -> Self { Self::new(parsed) }
		}
	};
}

raw_field_self!(i8);
raw_field_self!(u8);

raw_field_zerocopy!(zerocopy::F32<O>, f32);
raw_field_zerocopy!(zerocopy::F64<O>, f64);
raw_field_zerocopy!(zerocopy::I16<O>, i16);
raw_field_zerocopy!(zerocopy::I32<O>, i32);
raw_field_zerocopy!(zerocopy::I64<O>, i64);
raw_field_zerocopy!(zerocopy::I128<O>, i128);
raw_field_zerocopy!(zerocopy::Isize<O>, isize);
raw_field_zerocopy!(zerocopy::U16<O>, u16);
raw_field_zerocopy!(zerocopy::U32<O>, u32);
raw_field_zerocopy!(zerocopy::U64<O>, u64);
raw_field_zerocopy!(zerocopy::U128<O>, u128);
raw_field_zerocopy!(zerocopy::Usize<O>, usize);

pub trait EditableStruct {
	fn struct_name(&self) -> &str;
	fn number_of_fields(&self) -> usize;
	fn field_name(&self, index: usize) -> Option<&str>;
	fn field_ref(&self, index: usize) -> Option<&dyn std::any::Any>;
	fn field_mut(&mut self, index: usize) -> Option<&mut dyn std::any::Any>;
}
