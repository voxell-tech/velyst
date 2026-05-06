use bevy::prelude::*;
use bevy_vello::prelude::*;
use velyst::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_vello::VelloPlugin::default(),
            velyst::VelystPlugin,
        ))
        .register_typst_func::<MainFunc>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    commands.spawn((
        VelystFunc::new(
            asset_server.load("typst/box.typ"),
            MainFunc::default(),
        ),
        WorldScene::default(),
    ));
}

typst_func!(
    "main",
    #[derive(Default)]
    struct MainFunc {},
);
