use egui::KeyboardShortcut;

pub trait MenuAction: 'static {
	type Env;
	
	fn static_name(&self) -> &'static str;
	fn default_shortcut(&self) -> Option<KeyboardShortcut>;
	fn execute(&self, ctx: &egui::Context, env: &mut Self::Env);
}

pub struct RootMenu<A: MenuAction> {
	content: &'static [MenuItem<A>],
}

impl<A: MenuAction> RootMenu<A> {
	pub const fn new(content: &'static [MenuItem<A>]) -> Self {
		Self { content }
	}
	
	pub fn ui(&'static self, ui: &mut egui::Ui, env: &mut A::Env) {
		for item in self.content {
			item.ui(ui, env);
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
	CustomUi(fn(&mut egui::Ui, &mut A::Env)),
}

impl<A: MenuAction> MenuItem<A> {
	fn ui(&'static self, ui: &mut egui::Ui, env: &mut A::Env) {
		match self {
			Self::SubMenu(menu) => {
				ui.menu_button(menu.name, |ui| {
					for item in menu.content {
						item.ui(ui, env);
					}
				});
			},
			Self::Action(action) => {
				let mut button = egui::Button::new(action.static_name());
				if let Some(shortcut) = action.default_shortcut() {
					button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
				}
				if ui.add(button).clicked() {
					action.execute(ui.ctx(), env);
				}
			},
			Self::Separator => {
				ui.separator();
			},
			Self::CustomUi(ui_func) => {
				ui_func(ui, env);
			},
		}
	}
}
