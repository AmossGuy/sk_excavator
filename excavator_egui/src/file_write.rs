use std::path::PathBuf;
use std::sync::Arc;

use crate::file_read::{ItemInfo, BytesLoadResult};
use crate::plugins::{ItemLoaders, ThreadSpawner};

#[derive(Default)]
pub struct FileExtractor {
	queued: Vec<QueueItem>,
}

struct QueueItem {
	item: ItemInfo,
	dest: PathBuf,
}

impl FileExtractor {
	pub fn submit(&mut self, item: ItemInfo, dest: PathBuf) {
		self.queued.push(QueueItem { item, dest });
	}
	
	pub fn run(&mut self, ctx: &egui::Context) {
		let loaders = ctx.plugin_or_default::<ItemLoaders>();
		
		self.queued.retain(|thing| {
			match loaders.lock().bytes_loader.get_or_request(&thing.item, ctx) {
				None => true, // The queued thingy hasn't finished yet, keep it around to look at again later
				Some(result) => {
					if let Err(e) = Self::start_the_thing(ctx, result, thing) {
						println!("Error starting extraction: {}", e);
					};
					
					// We're done with this queue entry now
					false
				},
			}
		});
	}
	
	pub fn when_a_load_finishes(&mut self, ctx: &egui::Context, path: &PathBuf, result: Arc<BytesLoadResult>) {
		self.queued.retain(|thing| {
			if !(thing.item.outer_path() == path) {
				true // Not the one we're looking for, keep it
			} else {
				if let Err(e) = Self::start_the_thing(ctx, Arc::clone(&result), thing) {
					println!("Error starting extraction: {}", e);
				};
				false
			}
		});
	}
	
	fn start_the_thing(ctx: &egui::Context, load_result: Arc<BytesLoadResult>, thing: &QueueItem) -> Result<(), String> {
		let threads = ctx.plugin_or_default::<ThreadSpawner>();
		
		let bytes = crate::file_read::slice_item(load_result, &thing.item)?;
		let dest = thing.dest.clone();
		
		threads.lock().spawn(ctx.clone(), move |_| {
			if let Err(e) = std::fs::write(dest, bytes.as_slice()) {
				println!("Error during extraction: {}", e);
			};
			None
		});
		
		Ok(())
	}
}
