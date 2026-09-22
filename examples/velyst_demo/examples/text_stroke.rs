//! Renders the same stroked text through the vello and kanva
//! backends side by side, at several stroke thicknesses.

use bevy::prelude::*;
use velyst::bevy_vello::prelude::*;
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
    commands.spawn((
        Camera2d,
        VelloView,
        Camera {
            clear_color: Color::BLACK.into(),
            ..default()
        },
    ));

    let handle = asset_server.load("typst/text_stroke.typ");

    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|builder| {
            // Vello reference render.
            builder.spawn((
                VelystFunc::new(handle.clone(), MainFunc::default()),
                UiScene,
                Node {
                    width: percent(50.0),
                    height: percent(100.0),
                    ..default()
                },
            ));

            // Kanva render.
            builder.spawn((
                VelystFunc::new(handle, MainFunc::default()),
                UiScene,
                VelystKanva::default(),
                Node {
                    width: percent(50.0),
                    height: percent(100.0),
                    ..default()
                },
            ));
        });
}

typst_func!(
    "main",
    #[derive(Default)]
    struct MainFunc {},
    positional_args {},
);
