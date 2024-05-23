use bevy::prelude::*;
use bevy_typst::prelude::*;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        // Custom plugins
        .add_plugins(TypstPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (print_document, print_svg))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        asset_server.load::<TypstAsset>("hello_world.typ"),
        asset_server.load::<SvgAsset>("hello_world.typ"),
    ));
}

fn print_document(
    mut commands: Commands,
    q_typst_asset: Query<(Entity, &Handle<TypstAsset>)>,
    typst_asset: Res<Assets<TypstAsset>>,
) {
    let Ok((entity, handle)) = q_typst_asset.get_single() else {
        return;
    };

    if typst_asset.get(handle).is_some() {
        info!("Has document.");
        commands.entity(entity).remove::<Handle<TypstAsset>>();
    }
}

fn print_svg(
    mut commands: Commands,
    q_svg_asset: Query<(Entity, &Handle<SvgAsset>)>,
    svg_asset: Res<Assets<SvgAsset>>,
) {
    let Ok((entity, handle)) = q_svg_asset.get_single() else {
        return;
    };

    if svg_asset.get(handle).is_some() {
        info!("Has tree.");
        commands.entity(entity).remove::<Handle<SvgAsset>>();
    }
}
