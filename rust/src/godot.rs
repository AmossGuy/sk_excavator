mod autoload;
mod browser_tree;
mod format_resources;

use godot::prelude::*;

pub struct SkeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SkeExtension {}
