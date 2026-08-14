use excavator_backend::formats::{DropdownRenderer, EditableDataRenderer};
use bevy_ecs::{
	component::Component, entity::Entity, hierarchy::Children, system::Commands,
	world::{CommandQueue, EntityRef, World},
};

pub struct EntityTreeCallbacks {
	entity_ui: fn(&mut egui::Ui, EntityRef<'_>, &mut Commands),
}

impl EntityTreeCallbacks {
	pub const fn new() -> Self {
		Self {
			entity_ui: |_, _, _| {},
		}
	}
	
	pub const fn entity_ui(mut self, f: fn(&mut egui::Ui, EntityRef<'_>, &mut Commands)) -> Self {
		self.entity_ui = f;
		self
	}
}

pub fn entity_tree_ui(
	ui: &mut egui::Ui,
	world: &mut World, entity: Entity,
	callbacks: &EntityTreeCallbacks,
) {
	// Making our own queue because `World::commands` needs &mut World. Is there a better way?
	let mut queue = CommandQueue::default();
	let mut commands = Commands::new(&mut queue, world);
	
	entity_tree_ui_inner(ui, world, entity, callbacks, &mut commands);
	
	queue.apply(world);
}

fn entity_tree_ui_inner(
	ui: &mut egui::Ui,
	world: &World, entity: Entity,
	callbacks: &EntityTreeCallbacks,
	commands: &mut Commands,
) {
	let entity_ref = world.get_entity(entity).expect("entity should exist");
	let outer_size_option = entity_ref.get::<CachedOuterSize>();
	let inner_size_option = entity_ref.get::<CachedInnerSize>();
	
	let should_render_outer = outer_size_option
		.map(|outer_size| is_size_visible(ui, outer_size.size))
		.unwrap_or(true);
	
	if should_render_outer {
		let response = egui::containers::Frame::NONE.show(ui, |ui| {
			let should_render_inner = inner_size_option
				.map(|inner_size| is_size_visible(ui, inner_size.size))
				.unwrap_or(true);
			
			if should_render_inner {
				let response = egui::containers::Frame::group(ui.style()).show(ui, |ui| {
					ui.push_id(entity, |ui| {
						(callbacks.entity_ui)(ui, entity_ref, commands);
					});
				}).response;
				
				let new_inner_size = CachedInnerSize { size: response.rect.size() };
				if Some(&new_inner_size) != inner_size_option {
					commands.entity(entity).insert(new_inner_size);
				}
			} else {
				let inner_size = inner_size_option
					.expect("`should_render_inner` condition should ensure `inner_size_option` is Some in this branch");
				ui.allocate_space(inner_size.size);
			}
			
			let indent_frame = egui::containers::Frame::NONE.outer_margin(egui::Margin {
				left: 20,
				..egui::Margin::ZERO
			});
			let children_option = entity_ref.get::<Children>();
			
			indent_frame.show(ui, |ui| {
				for child in children_option.into_iter().flatten().copied() {
					entity_tree_ui_inner(ui, world, child, callbacks, commands);
				}
			});
		}).response;
		
		let new_outer_size = CachedOuterSize { size: response.rect.size() };
		if Some(&new_outer_size) != outer_size_option {
			commands.entity(entity).insert(new_outer_size);
		}
	} else {
		let outer_size = outer_size_option
			.expect("`should_render_outer` condition should ensure `outer_size_option` is Some in this branch");
		ui.allocate_space(outer_size.size);
	}
}

fn is_size_visible(ui: &egui::Ui, size: egui::Vec2) -> bool {
	let next_widget_position = ui.next_widget_position();
	ui.is_rect_visible(
		egui::Rect::from_min_size(next_widget_position, size),
	)
}

pub struct EguiDataRenderer<'a> {
	pub ui: &'a mut egui::Ui,
}

impl EditableDataRenderer for EguiDataRenderer<'_> {
	type Dropdown<'a> = EguiDropdownRenderer<'a>;
	
	fn dropdown(&mut self, name: &str, selected_text: &str, contents: impl FnOnce(Self::Dropdown<'_>)) {
		let ui = &mut self.ui;
		ui.label(name);
		egui::ComboBox::from_id_salt(name)
			.selected_text(selected_text)
			.show_ui(ui, |ui| {
				contents(EguiDropdownRenderer { ui })
			});
		ui.end_row();
	}
	
	fn field_f32(&mut self, name: &str, value: &mut f32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u16(&mut self, name: &str, value: &mut u16) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
	
	fn field_u32(&mut self, name: &str, value: &mut u32) {
		let ui = &mut self.ui;
		ui.label(name);
		ui.add(egui::DragValue::new(value));
		ui.end_row();
	}
}

pub struct EguiDropdownRenderer<'a> {
	pub ui: &'a mut egui::Ui,
}

impl DropdownRenderer for EguiDropdownRenderer<'_> {
	fn choice(&mut self, name: &str, selected: bool) -> bool {
		let ui = &mut self.ui;
		ui.selectable_label(selected, name).clicked()
	}
}

#[derive(Component, PartialEq)]
struct CachedOuterSize {
	size: egui::Vec2,
}

#[derive(Component, PartialEq)]
struct CachedInnerSize {
	size: egui::Vec2,
}
