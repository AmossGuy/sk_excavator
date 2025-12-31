mod autoload;
mod files;

use godot::prelude::*;

struct SkeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SkeExtension {}
