use std::sync::Arc;
use yoke::Yoke;


// need to refurbish excavator_egui::file_read::ItemLoader before we can make this not silly
type GlueBytesLoadResult = Result<Box<[u8]>, Box<str>>;

#[derive(Clone)]
pub struct FileBytes {
	yoke: Yoke<&'static [u8], Arc<GlueBytesLoadResult>>,
}

impl FileBytes {
	// need to refurbish excavator_egui::file_read::ItemLoader before we can make this not silly
	pub fn glue_new(cart: Arc<GlueBytesLoadResult>, range: impl std::slice::SliceIndex<[u8], Output = [u8]>) -> Self {
		let yoke = Yoke::attach_to_cart(cart, |load_result| {
			// "so like if we've gotten this far we've already checked the result is ok" (regarding the unwrap)
			&load_result.as_ref().unwrap()[range]
		});
		Self { yoke }
	}
	
	pub fn cropped(self, range: impl std::slice::SliceIndex<[u8], Output = [u8]>) -> Option<Self> {
		let yoke_result = self.yoke.try_map_project(|bytes, _| {
			bytes.get(range).ok_or(())
		});
		yoke_result.ok().map(|yoke| Self { yoke })
	}
}

impl std::ops::Deref for FileBytes {
	type Target = [u8];
	
	fn deref(&self) -> &[u8] {
		self.yoke.get()
	}
}
