use godot::prelude::*;

mod animators;
mod second_order_systems;

/// DGExtension entry
struct GodotSecondOrderAnimationsExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotSecondOrderAnimationsExtension {}
