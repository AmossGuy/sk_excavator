use std::sync::Arc;
use thunderdome::Arena;

use crate::app::context::ExcavatorContext;

pub trait Window: Send + 'static {
	fn ui(&mut self, ui: &mut egui::Ui, excavator: &ExcavatorContext);
	
	fn build_viewport(&self, builder: &mut egui::ViewportBuilder) {
		let _ = builder;
	}
}

pub struct WindowHolder {
	id: egui::Id,
	windows: Arena<Arc<egui::mutex::Mutex<WindowHolderWindow>>>,
}

struct WindowHolderWindow {
	window: Box<dyn Window>,
	wants_to_close: bool,
}

impl WindowHolder {
	pub fn new(id_source: impl egui::AsId) -> Self {
		Self {
			id: egui::Id::new(id_source),
			windows: Arena::new(),
		}
	}
	
	pub fn add(&mut self, window: Box<dyn Window>) {
		self.windows.insert(Arc::new(egui::mutex::Mutex::new(
			WindowHolderWindow { window, wants_to_close: false },
		)));
	}
	
	pub fn show_all(&mut self, ctx: &egui::Context, excavator: &ExcavatorContext) {
		// The retain method's ability to remove elements is infrequently used here. Mostly we just want to loop over all the windows so we can display them.
		self.windows.retain(|arena_index, window| {
			let window_lock = window.lock();
			
			if window_lock.wants_to_close {
				// The uncommon case: close the window
				return false;
			}
			
			let viewport_id = egui::ViewportId(self.id.with(arena_index));
			let mut builder = egui::ViewportBuilder::default();
			window_lock.window.build_viewport(&mut builder);
			
			let window = window.clone();
			let excavator = excavator.clone();
			
			// Remember: the code inside and outside this closure runs at different times!
			ctx.show_viewport_deferred(viewport_id, builder, move |ui, _class| {
				let mut window_lock = window.lock();
				
				window_lock.window.ui(ui, &excavator);
				
				if ui.ctx().input(|state| state.viewport().close_requested()) {
					window_lock.wants_to_close = true;
					excavator.set_needs_parent_repaint();
				}
				
				excavator.repaint_parent_if_needed(ui.ctx());
			});
			
			// The common case: don't close the window
			true
		});
	}
}
