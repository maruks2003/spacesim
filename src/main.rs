use crate::physics_plugin::PhysicsPlugin;
use bevy::prelude::*;

mod physics_plugin;
mod quadtree;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .run();
}
