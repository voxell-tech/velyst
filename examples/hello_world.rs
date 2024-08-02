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
        .add_systems(Update, (print_document, print_svg))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        asset_server.load::<TypstAsset>("hello_world.typ"),
        asset_server.load::<SvgAsset>("hello_world.typ"),
        VelloAssetBundle {
            vector: asset_server.load("hello_world.typ"),
            ..default()
        },
    ));
}

fn print_document(
    mut commands: Commands,
    q_typst_asset: Query<(Entity, &Handle<TypstAsset>)>,
    typst_assets: Res<Assets<TypstAsset>>,
) {
    let Ok((entity, handle)) = q_typst_asset.get_single() else {
        return;
    };

    if typst_assets.get(handle).is_some() {
        info!("Has document.");
        commands.entity(entity).remove::<Handle<TypstAsset>>();
    }
}

fn print_svg(
    mut commands: Commands,
    q_svg_asset: Query<(Entity, &Handle<SvgAsset>)>,
    svg_assets: Res<Assets<SvgAsset>>,
) {
    let Ok((entity, handle)) = q_svg_asset.get_single() else {
        return;
    };

    if svg_assets.get(handle).is_some() {
        info!("Has tree.");
        commands.entity(entity).remove::<Handle<SvgAsset>>();
    }
}
