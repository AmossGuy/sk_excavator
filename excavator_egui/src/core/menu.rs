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

pub enum MenuItem<A: MenuAction> {
	SubMenu(&'static str, &'static [MenuItem<A>]),
	Action(A),
	Separator,
	CustomCondition(fn(&egui::Context, &mut A::Env) -> bool, &'static [MenuItem<A>]),
	CustomUi(fn(&mut egui::Ui, &mut A::Env)),
}

impl<A: MenuAction> MenuItem<A> {
	fn ui(&'static self, ui: &mut egui::Ui, env: &mut A::Env) {
		match self {
			Self::SubMenu(name, content) => {
				ui.menu_button(*name, |ui| {
					// egui's popup sizing stinks
					// with this workaround, we still have the problem that menus will never shrink horizontally, but at least it'll avoid text wrapping in weird ways
					ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
					
					for item in *content {
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
			Self::CustomCondition(condition, content) => {
				if condition(ui.ctx(), env) {
					for item in content.iter() {
						item.ui(ui, env);
					}
				}
			},
			Self::CustomUi(ui_func) => {
				ui_func(ui, env);
			},
		}
	}
}
