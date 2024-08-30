use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_typst::prelude::*;
use bevy_vello::{integrations::VelloAsset, VelloAssetBundle, VelloPlugin};
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
        asset_server.load::<TypstModAsset>("hello_world.typ"),
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
    q_typst_asset: Query<(Entity, &Handle<TypstModAsset>)>,
    typst_mod_assets: Res<Assets<TypstModAsset>>,
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
        commands.entity(entity).remove::<Handle<TypstModAsset>>();
    }
}

fn pan_and_zoom(
    window: Query<&Window, With<PrimaryWindow>>,
    mut vello_asset: Query<&mut Transform, With<Handle<VelloAsset>>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut start_translation: Local<Vec2>,
    mut start_cursor: Local<Vec2>,
    mut evr_scroll: EventReader<MouseWheel>,
) {
    let Ok(mut vello_transform) = vello_asset.get_single_mut() else {
        return;
    };
    let Ok(Some(cursor_position)) = window.get_single().map(|w| w.cursor_position()) else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        *start_translation = vello_transform.translation.xy();
        *start_cursor = cursor_position;
    }

    // Pan as long as mouse left is being pressed
    if mouse.pressed(MouseButton::Left) {
        let mut offset = cursor_position - *start_cursor;
        offset.y = -offset.y;
        let translation = *start_translation + offset;

        vello_transform.translation.x = translation.x;
        vello_transform.translation.y = translation.y;
    }

    const SCALE_FACTOR: f32 = 0.12;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                vello_transform.scale += Vec3::splat(SCALE_FACTOR * ev.y * 10.0);
            }
            MouseScrollUnit::Pixel => {
                vello_transform.scale += Vec3::splat(SCALE_FACTOR * ev.y);
            }
        }
    }
}
