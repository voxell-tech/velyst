use bevy::prelude::*;
use bevy_typst::prelude::*;
use bevy_vello::{VelloAssetBundle, VelloPlugin};

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        // Custom plugins
        .add_plugins((TypstPlugin::default(), VelloPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((VelloAssetBundle {
        vector: asset_server.load("simple_ui.typ"),
        ..default()
    },));
}
