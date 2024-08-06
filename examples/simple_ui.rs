use bevy::{prelude::*, window::PrimaryWindow};
use bevy_typst::{
    compiler::{TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};
use typst_element::prelude::*;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        // Custom plugins
        .add_plugins((TypstPlugin::default(), VelloPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, ui_update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((VelloAssetBundle {
        vector: asset_server.load("simple_ui.typ"),
        ..default()
    },));

    commands.spawn(VelloSceneBundle::default());
}

fn ui_update(
    mut q_scene: Query<(&mut VelloScene, &mut Transform)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    world: Res<TypstCompiler>,
    time: Res<Time>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let width = window.width() as f64;
    let height = window.height() as f64;

    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let Ok((mut scene, mut transform)) = q_scene.get_single_mut() else {
        return;
    };

    let mut writer = SimpleWriter::new();

    writer.blank_page(width, height, |writer| {
        writer.add_content(
            align(sequence!(
                heading(text(time.elapsed_seconds().to_string())),
                text((1.0 / time.delta_seconds()).to_string())
            ))
            .with_alignment(layout::Alignment::Both(
                layout::HAlignment::Center,
                layout::VAlignment::Horizon,
            )),
        );
    });

    let document = world.compile_content(writer.pack()).unwrap();
    let typst_scene = TypstScene::from_document(&document, Abs::zero()).unwrap();

    *transform = Transform::from_xyz(-typst_scene.width * 0.5, typst_scene.height * 0.5, 0.0);
    *scene = typst_scene.as_component();
}
