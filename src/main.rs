use bevy::prelude::*;

mod space_floaty;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(space_floaty::SpaceFloaty)
        .run();
}
