mod autoload;

use godot::prelude::*;

pub struct SkeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SkeExtension {}
