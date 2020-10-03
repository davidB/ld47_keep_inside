use bevy::prelude::*;

fn main() {
    App::build()
        .add_default_plugins()
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}
