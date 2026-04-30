use super::app::ExcavatorApp;
use super::menubar::MenuBarAction;

#[derive(Debug)]
pub enum Message {
	MenuBarAction(MenuBarAction),
}

impl Message {
	fn apply(self, ctx: &egui::Context, app: &mut ExcavatorApp) {
		match self {
			Self::MenuBarAction(action) => action.apply(ctx, app),
		}
	}
}

pub fn send_message(ctx: &egui::Context, message: Message) {
	let plugin = ctx.plugin_or_default::<MessageQueue>();
	plugin.lock().messages.push(message);
}

pub fn apply_messages(ctx: &egui::Context, app: &mut ExcavatorApp) {
	if let Some(plugin) = ctx.plugin_opt::<MessageQueue>() {
		for message in plugin.lock().messages.drain(..) {
			message.apply(ctx, app);
		}
	}
}

#[derive(Default)]
struct MessageQueue {
	messages: Vec<Message>,
}

impl egui::Plugin for MessageQueue {
	fn debug_name(&self) -> &'static str {
		"MessageQueue (excavator)"
	}
}

pub fn show_status_bar_panel(ui: &mut egui::Ui) {
	egui::Panel::bottom("status bar").show_inside(ui, |_ui| {
		// TODO
	});
}
