mod autoload;
mod file_tree;

use godot::prelude::*;

pub struct SkeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SkeExtension {}
