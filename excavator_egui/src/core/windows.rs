use std::sync::{Arc, Mutex};
use super::menubar::show_menu_bar_panel;
use super::message::show_status_bar_panel;

#[derive(Default)]
pub struct WindowHolder {
	windows: Vec<Option<DynWrappedWindow>>,
}

struct LiveWindow<W: ?Sized> {
	state: LiveWindowState,
	// Dynamically sized... playing with fire here, but it's still good.
	itself: W,
}

type WrappedWindow<W> = Arc<Mutex<LiveWindow<W>>>;
type DynWrappedWindow = WrappedWindow<dyn Window>;

impl WindowHolder {
	pub fn new() -> Self {
		Self::default()
	}
	
	pub fn add(&mut self, window: impl Window) {
		let window = Arc::new(Mutex::new(LiveWindow {
			state: LiveWindowState::default(),
			itself: window,
		}));
		self.add_dyn(window);
	}
	
	fn add_dyn(&mut self, window: DynWrappedWindow) {
		// If windows has an entry that's None, put the new window there. Otherwise, push it at the end.
		if let Some(slot) = self.windows.iter_mut().find(|x| x.is_none()) {
			*slot = Some(window);
		} else {
			self.windows.push(Some(window));
		}
	}
	
	pub fn show_as_viewports(&mut self, ui: &mut egui::Ui) {
		let base_id = ui.id().with("WindowHolder");
		for (i, window_opt) in self.windows.iter_mut().enumerate() {
			if let Some(window) = window_opt {
				let parent_id = ui.viewport_id();
				let viewport_id = egui::ViewportId(base_id.with(i));
				let builder = egui::ViewportBuilder::default();
				let window_clone = Arc::clone(window);
				
				ui.show_viewport_deferred(viewport_id, builder, move |ui, class| {
					Self::viewport_callback(&window_clone, ui, class, parent_id);
				});
				
				if window.lock().unwrap().state.doomed {
					*window_opt = None;
				}
			}
		}
	}
	
	fn viewport_callback(window_m: &DynWrappedWindow, ui: &mut egui::Ui, _class: egui::ViewportClass, parent_id: egui::ViewportId) {
		let mut window = window_m.lock().unwrap();
		let settings = window.itself.settings();
		
		if settings.show_menubar { show_menu_bar_panel(ui); }
		if settings.show_statusbar { show_status_bar_panel(ui); }
		
		window.itself.ui(ui);
		
		if ui.ctx().input(|state| state.viewport().close_requested()) {
			window.state.doomed = true;
		}
	}
}

pub trait Window: Send + 'static {
	fn ui(&mut self, ui: &mut egui::Ui);
	
	fn settings(&self) -> WindowSettings {
		WindowSettings::default()
	}
}

#[derive(Clone)]
pub struct WindowSettings {
	show_menubar: bool,
	show_statusbar: bool,
	resizable: bool,
}

impl Default for WindowSettings {
	fn default() -> Self {
		Self {
			show_menubar: false,
			show_statusbar: false,
			resizable: true,
		}
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
