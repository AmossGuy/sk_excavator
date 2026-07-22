use zerocopy::byteorder::*;

pub trait RawField {
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

raw_field_zerocopy!(F32<O>, f32);
raw_field_zerocopy!(F64<O>, f64);
raw_field_zerocopy!(I16<O>, i16);
raw_field_zerocopy!(I32<O>, i32);
raw_field_zerocopy!(I64<O>, i64);
raw_field_zerocopy!(I128<O>, i128);
raw_field_zerocopy!(Isize<O>, isize);
raw_field_zerocopy!(U16<O>, u16);
raw_field_zerocopy!(U32<O>, u32);
raw_field_zerocopy!(U64<O>, u64);
raw_field_zerocopy!(U128<O>, u128);
raw_field_zerocopy!(Usize<O>, usize);
