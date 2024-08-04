use bevy::{prelude::*, window::PrimaryWindow};
use bevy_typst::{
    compiler::{TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{VelloAssetBundle, VelloPlugin, VelloScene, VelloSceneBundle};
use typst::{
    foundations::{Content, FromValue, NativeElement, SequenceElem, Smart},
    layout::{
        Abs, AlignElem, Alignment, BlockElem, BoxElem, HAlignment, Length, Margin, PageElem, Ratio,
        Rel, VAlignment,
    },
    model::HeadingElem,
    text::TextElem,
};

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
    // commands.spawn((VelloAssetBundle {
    //     vector: asset_server.load("simple_ui.typ"),
    //     ..default()
    // },));

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

    let width = window.width() as f64 / 1.3333334;
    let height = window.height() as f64 / 1.3333334;

    println!("width: {width}, height: {height}");

    let Ok((mut scene, mut transform)) = q_scene.get_single_mut() else {
        return;
    };

    let content = PageElem::new(
        SequenceElem::new(vec![
            HeadingElem::new(TextElem::new(time.elapsed_seconds().to_string().into()).pack())
                .pack(),
            TextElem::new((1.0 / time.delta_seconds()).to_string().into()).pack(),
        ])
        .pack(),
    )
    .with_width(Smart::Custom(Length::from(Abs::pt(width))))
    .with_height(Smart::Custom(Length::from(Abs::pt(height))))
    .with_margin(Margin::splat(Some(Smart::Custom(Rel::zero()))))
    .pack()
    .styled(AlignElem::set_alignment(Alignment::Both(
        HAlignment::Center,
        VAlignment::Horizon,
    )));

    let mut document = world.compile_content(content).unwrap();
    let frame = std::mem::take(&mut document.pages[0].frame);
    // println!("{:?}", frame.width().to_raw());
    document.pages[0].frame = frame.mark_box();
    let typst_scene = TypstScene::from_document(&document, Abs::zero()).unwrap();

    println!(
        "scene: width: {}, height: {}",
        typst_scene.width, typst_scene.height
    );

    *transform = Transform::from_xyz(-typst_scene.width * 0.5, typst_scene.height * 0.5, 0.0);
    *scene = typst_scene.as_component();
}
