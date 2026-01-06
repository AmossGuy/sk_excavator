mod autoload;
pub mod browser_tree;
pub mod file_view;
mod format_resources;

use godot::prelude::*;

pub struct SkeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SkeExtension {}
