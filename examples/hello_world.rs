use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_typst::prelude::*;
use bevy_vello::{VelloAssetBundle, VelloPlugin};
use typst::foundations::Label;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(TypstPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (check_document, check_module, pan_and_zoom))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        asset_server.load::<TypstDocAsset>("hello_world.typ"),
        asset_server.load::<TypstAsset>("hello_world.typ"),
        VelloAssetBundle {
            asset: asset_server.load("hello_world.typ"),
            ..default()
        },
    ));
}

fn check_document(
    mut commands: Commands,
    q_typst_asset: Query<(Entity, &Handle<TypstDocAsset>)>,
    typst_doc_assets: Res<Assets<TypstDocAsset>>,
) {
    let Ok((entity, handle)) = q_typst_asset.get_single() else {
        return;
    };

    if typst_doc_assets.get(handle).is_some() {
        info!("Has document.");
        commands.entity(entity).remove::<Handle<TypstDocAsset>>();
    }
}

fn check_module(
    mut commands: Commands,
    q_typst_asset: Query<(Entity, &Handle<TypstAsset>)>,
    typst_mod_assets: Res<Assets<TypstAsset>>,
) {
    let Ok((entity, handle)) = q_typst_asset.get_single() else {
        return;
    };

    if let Some(module) = typst_mod_assets.get(handle).map(|asset| asset.module()) {
        info!("Has module.");
        let title_label =
            module
                .clone()
                .content()
                .query_first(typst::foundations::Selector::Label(Label::new(
                    "title-label",
                )));
        println!("title-label: {title_label:?}");
        commands.entity(entity).remove::<Handle<TypstAsset>>();
    }
}

fn pan_and_zoom(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&mut OrthographicProjection, &mut Transform), With<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut start_translation: Local<Vec2>,
    mut start_cursor: Local<Vec2>,
    mut evr_scroll: EventReader<MouseWheel>,
) {
    let Ok((mut projection, mut transform)) = q_camera.get_single_mut() else {
        return;
    };
    let Ok(Some(cursor_position)) = q_window.get_single().map(|w| w.cursor_position()) else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        *start_translation = transform.translation.xy();
        *start_cursor = cursor_position;
    }

    // Pan as long as mouse left is being pressed
    if mouse.pressed(MouseButton::Left) {
        let mut offset = cursor_position - *start_cursor;
        offset.x = -offset.x;
        let translation = *start_translation + offset * projection.scale;

        transform.translation.x = translation.x;
        transform.translation.y = translation.y;
    }

    const SCALE_FACTOR: f32 = 0.012;
    for ev in evr_scroll.read() {
        let scale_offset = match ev.unit {
            MouseScrollUnit::Line => SCALE_FACTOR * ev.y * 10.0,
            MouseScrollUnit::Pixel => SCALE_FACTOR * ev.y,
        };

        projection.scale -= projection.scale * scale_offset;
    }
}
