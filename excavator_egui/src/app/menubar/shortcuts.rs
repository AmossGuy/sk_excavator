use egui::{Key, KeyboardShortcut, Modifiers};
use std::{cmp, collections::HashMap};

use super::MenuAction;

pub struct ShortcutStorage {
	for_format: HashMap<MenuAction, KeyboardShortcut>,
	// sorted in a particular way
	for_consume: Vec<(KeyboardShortcut, MenuAction)>,
}

impl ShortcutStorage {
	pub fn new() -> Self {
		type KS = KeyboardShortcut;
		type Mod = Modifiers;
		
		let for_format = HashMap::from([
			(MenuAction::OpenFile, KS::new(Mod::COMMAND, Key::O)),
			(MenuAction::Save, KS::new(Mod::COMMAND, Key::S)),
			(MenuAction::SaveAs, KS::new(Mod::COMMAND | Mod::SHIFT, Key::S)),
			(MenuAction::Quit, KS::new(Mod::COMMAND, Key::Q)),
			
			(MenuAction::Undo, KS::new(Mod::COMMAND, Key::Z)),
			(MenuAction::Redo, KS::new(Mod::COMMAND | Mod::SHIFT, Key::Z)),
		]);
		
		let mut for_consume = for_format.iter()
			.map(|(&action, &shortcut)| (shortcut, action))
			.collect::<Vec<_>>();
		
		// Sort from most to least modifiers, because in egui, consuming a shortcut with fewer modifiers first prevents a similar shortcut with a superset of modifiers from registering later
		for_consume.sort_unstable_by_key(|(shortcut, _)| {
			cmp::Reverse(count_modifiers(shortcut.modifiers))
		});
		
		Self { for_format, for_consume }
	}
	
	pub(super) fn get_action_shortcut(&self, action: MenuAction) -> Option<KeyboardShortcut> {
		self.for_format.get(&action).copied()
	}
	
	pub(super) fn test_shortcuts(&self, ctx: &egui::Context) -> Option<MenuAction> {
		ctx.input_mut(|input| {
			for (shortcut, action) in &self.for_consume {
				if input.consume_shortcut(shortcut) {
					return Some(*action);
				}
			}
			None
		})
	}
}

fn count_modifiers(modifiers: Modifiers) -> u8 {
	[modifiers.alt, modifiers.ctrl, modifiers.shift, modifiers.mac_cmd, modifiers.command]
		.iter().copied().map(u8::from).sum()
}
