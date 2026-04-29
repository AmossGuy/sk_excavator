use egui::KeyboardShortcut;

pub trait MenuAction: 'static {
	fn static_name(&self) -> &'static str;
	fn default_shortcut(&self) -> Option<KeyboardShortcut>;
}

pub struct RootMenu<A: MenuAction> {
	content: &'static [MenuItem<A>],
}

impl<A: MenuAction> RootMenu<A> {
	pub const fn new(content: &'static [MenuItem<A>]) -> Self {
		Self { content }
	}
	
	pub fn ui(&self, ui: &mut egui::Ui) {
		for item in self.content {
			item.ui(ui);
		}
	}
}

pub struct Menu<A: MenuAction> {
	name: &'static str,
	content: &'static [MenuItem<A>],
}

impl<A: MenuAction> Menu<A> {
	pub const fn new(name: &'static str, content: &'static [MenuItem<A>]) -> Self {
		Self { name, content }
	}
}

pub enum MenuItem<A: MenuAction> {
	SubMenu(Menu<A>),
	Action(A),
	Separator,
}

impl<A: MenuAction> MenuItem<A> {
	fn ui(&self, ui: &mut egui::Ui) {
		match self {
			Self::SubMenu(menu) => {
				ui.menu_button(menu.name, |ui| {
					for item in menu.content {
						item.ui(ui);
					}
				});
			},
			Self::Action(action) => {
				let mut button = egui::Button::new(action.static_name());
				if let Some(shortcut) = action.default_shortcut() {
					button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
				}
				ui.add(button);
			},
			Self::Separator => {
				ui.separator();
			},
		}
	}
}
