use bevy::prelude::*;
use bevy_vello::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_vello::VelloPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    commands.spawn(VelloSvgHandle(
        asset_server.load("images/voxell_logo.svg"),
    ));
}
