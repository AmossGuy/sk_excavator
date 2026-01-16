mod autoload;
pub mod browser_tree;
pub mod file_view;
pub mod file_view_st;
mod format_resources;

use godot::prelude::*;

pub struct SkeExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SkeExtension {}
