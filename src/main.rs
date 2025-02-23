use bevy::prelude::*;
use bevy_chess::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
