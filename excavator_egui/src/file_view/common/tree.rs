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
	let Some(tree_component) = entity_ref.get::<ShowInTree>() else { return };
	
	let mut new_outer_height: Option<f32> = None;
	let mut new_inner_height: Option<f32> = None;
	
	let should_render_outer = tree_component.cached_height_outer
		.map(|height| is_height_visible(ui, height))
		.unwrap_or(true);
	
	if should_render_outer {
		let response = egui::containers::Frame::NONE.show(ui, |ui| {
			let should_render_inner = tree_component.cached_height_inner
				.map(|height| is_height_visible(ui, height))
				.unwrap_or(true);
			
			if should_render_inner {
				let response = egui::containers::Frame::group(ui.style()).show(ui, |ui| {
					ui.push_id(entity, |ui| {
						(callbacks.entity_ui)(ui, entity_ref, commands);
					});
				}).response;
				
				new_inner_height = Some(response.rect.height());
			} else {
				let height = tree_component.cached_height_inner
					.expect("`should_render_inner` condition should ensure `cached_height_inner` is Some in this branch");
				ui.allocate_space(egui::Vec2::new(1.0, height));
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
		
		new_outer_height = Some(response.rect.height());
	} else {
		let height = tree_component.cached_height_outer
			.expect("`should_render_outer` condition should ensure `outer_size_option` is Some in this branch");
		ui.allocate_space(egui::Vec2::new(1.0, height));
	}
	
	if new_outer_height.is_some() || new_inner_height.is_some() {
		let new_tree_component = ShowInTree {
			cached_height_outer: new_outer_height.or(tree_component.cached_height_outer),
			cached_height_inner: new_inner_height.or(tree_component.cached_height_inner),
		};
		commands.entity(entity).insert(new_tree_component);
	}
}

fn is_height_visible(ui: &egui::Ui, height: f32) -> bool {
	let next_widget_position = ui.next_widget_position();
	ui.is_rect_visible(egui::Rect::from_min_size(
		next_widget_position,
		egui::Vec2::new(1.0, height),
	))
}

#[derive(Component, Default)]
pub struct ShowInTree {
	cached_height_outer: Option<f32>,
	cached_height_inner: Option<f32>,
}
