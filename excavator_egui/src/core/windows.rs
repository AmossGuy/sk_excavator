use std::sync::{Arc, Mutex};
use crate::core::app::ExcavatorContext;

pub struct WindowHolder {
	windows: Vec<Option<WrappedDynWindow>>,
}

struct LiveWindow<W: ?Sized> {
	state: LiveWindowState,
	// Dynamically sized... playing with fire here, but it's still good.
	itself: W,
}

type WrappedWindow<W> = Arc<Mutex<LiveWindow<W>>>;
type WrappedDynWindow = WrappedWindow<dyn Window>;

impl WindowHolder {
	pub fn new() -> Self {
		Self { windows: Vec::new() }
	}
	
	pub fn add(&mut self, window: impl Window) {
		let window = Arc::new(Mutex::new(LiveWindow {
			state: LiveWindowState::default(),
			itself: window,
		}));
		self.add_dyn(window);
	}
	
	fn add_dyn(&mut self, window: WrappedDynWindow) {
		// If windows has an entry that's None, put the new window there. Otherwise, push it at the end.
		if let Some(slot) = self.windows.iter_mut().find(|x| x.is_none()) {
			*slot = Some(window);
		} else {
			self.windows.push(Some(window));
		}
	}
	
	pub fn show_as_viewports(&mut self, ui: &mut egui::Ui, excavator: &ExcavatorContext) {
		let base_id = ui.id().with("WindowHolder");
		for (i, window_opt) in self.windows.iter_mut().enumerate() {
			if let Some(window) = window_opt {
				let initial_size = window.lock().unwrap().itself.initial_size();
				
				let viewport_id = egui::ViewportId(base_id.with(i));
				let builder = egui::ViewportBuilder::default().with_inner_size(initial_size);
				let window_clone = Arc::clone(window);
				let excavator_clone = excavator.clone();
				
				ui.show_viewport_deferred(viewport_id, builder, move |ui, _class| {
					Self::viewport_callback(&window_clone, ui, &excavator_clone);
				});
				
				if window.lock().unwrap().state.doomed {
					*window_opt = None;
				}
			}
		}
	}
	
	fn viewport_callback(window_m: &WrappedDynWindow, ui: &mut egui::Ui, excavator: &ExcavatorContext) {
		let mut window = window_m.lock().unwrap();
		
		window.itself.ui(ui, excavator);
		
		if ui.ctx().input(|state| state.viewport().close_requested()) {
			window.state.doomed = true;
		}
	}
}

pub trait Window: Send + 'static {
	fn ui(&mut self, ui: &mut egui::Ui, excavator: &ExcavatorContext);
	
	fn initial_size(&self) -> egui::Vec2 {
		egui::Vec2::splat(300.0)
	}
}

struct LiveWindowState {
	doomed: bool,
}

impl Default for LiveWindowState {
	fn default() -> Self {
		Self {
			doomed: false,
		}
	}
}
