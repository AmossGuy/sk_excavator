use std::thread::JoinHandle;
use crate::{ExcavatorApp, ExcavatorMessage};

#[derive(Default)]
pub struct MessageQueue {
	messages: Vec<ExcavatorMessage>,
}

impl egui::Plugin for MessageQueue {
	fn debug_name(&self) -> &'static str {
		"MessageQueue (excavator)"
	}
}

impl MessageQueue {
	pub fn send(&mut self, message: ExcavatorMessage) {
		self.messages.push(message);
	}
	
	pub fn send_multiple(&mut self, messages: impl Iterator<Item = ExcavatorMessage>) {
		self.messages.extend(messages);
	}
	
	pub fn apply_all(&mut self, app: &mut ExcavatorApp, ctx: &egui::Context) {
		for message in self.messages.drain(..) {
			message.apply(app, ctx);
		}
	}
}

#[derive(Default)]
pub struct ThreadSpawner {
	handles: Vec<JoinHandle<Option<ExcavatorMessage>>>,
}

impl egui::Plugin for ThreadSpawner {
	fn debug_name(&self) -> &'static str {
		"ThreadSpawner (excavator)"
	}
}

impl ThreadSpawner {
	pub fn spawn<F>(&mut self, ctx: egui::Context, f: F)
	where
		F: FnOnce(&egui::Context) -> Option<ExcavatorMessage>,
		F: Send + 'static,
	{
		println!("ThreadSpawner: spawning thread");
		let handle = std::thread::spawn(move || {
			let message = f(&ctx);
			if message.is_some() {
				ctx.request_repaint();
			}
			println!("ThreadSpawner: thread ending");
			message
		});
		self.handles.push(handle);
	}
	
	pub fn take_messages(&mut self) -> impl Iterator<Item = ExcavatorMessage> {
		self.handles
			.extract_if(.., |h| h.is_finished())
			.filter_map(|h| match h.join() {
				Ok(message_option) => message_option,
				Err(e) => std::panic::resume_unwind(e),
			})
	}
}
