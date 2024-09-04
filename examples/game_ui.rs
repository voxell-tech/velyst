use std::marker::PhantomData;

use bevy::{color::palettes::css, prelude::*, window::PrimaryWindow};
use bevy_typst::{prelude::*, TypstPlugin};
use bevy_vello::{prelude::*, VelloPlugin};
use typst::World;
use typst_element::prelude::*;
use typst_vello::TypstScene;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(TypstPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, root_ui)
        .add_systems(
            Update,
            perf_metrics.run_if(resource_exists::<UiContext<RootUi>>),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..default()
    });

    // commands
    //     .spawn(ButtonBundle {
    //         style: Style {
    //             position_type: PositionType::Absolute,
    //             width: Val::Px(100.0),
    //             height: Val::Px(100.0),
    //             ..default()
    //         },
    //         background_color: css::RED.into(),
    //         ..default()
    //     })
    //     .with_children(|builder| {
    //         builder.spawn(ButtonBundle {
    //             style: Style {
    //                 position_type: PositionType::Absolute,
    //                 width: Val::Px(50.0),
    //                 height: Val::Px(50.0),
    //                 ..default()
    //             },
    //             background_color: css::BLUE.into(),
    //             ..default()
    //         });
    //     });
}

fn perf_metrics(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_scene: Query<&mut VelloScene>,
    root_ui: Res<UiContext<RootUi>>,
    world: Res<TypstCompiler>,
    mod_assets: Res<Assets<TypstModAsset>>,
    time: Res<Time>,
) {
    let Ok(window_size) = q_window.get_single().map(|w| w.size()) else {
        return;
    };
    let Ok(mut scene) = q_scene.get_single_mut() else {
        return;
    };
    let Some(scope) = mod_assets.get(root_ui.module()).map(|m| m.module().scope()) else {
        return;
    };

    let content = context(scope.get_unchecked_func("main"), |args| {
        args.push_named("main_width", Abs::pt(window_size.x as f64));
        args.push_named("main_height", Abs::pt(window_size.y as f64));
        args.push_named("fps", 1.0 / time.delta_seconds_f64());
        args.push_named("elapsed_time", time.elapsed_seconds_f64());
    })
    .pack();

    let frame = world
        .scoped_engine(|engine| {
            let locator = typst::introspection::Locator::root();
            let styles = foundations::StyleChain::new(&world.library().styles);

            typst::layout::layout_frame(
                engine,
                &content,
                locator,
                styles,
                layout::Region::new(
                    layout::Axes::new(Abs::inf(), Abs::inf()),
                    layout::Axes::new(false, false),
                ),
            )
        })
        .unwrap();

    // let frame = &world.compile_content(&content).unwrap().pages[0].frame;

    let mut typst_scene = TypstScene::from_frame(&frame);
    *scene = VelloScene::from(typst_scene.render());
}

#[derive(Resource, Default)]
pub struct UiContext<T> {
    // scene: TypstScene,
    module: Handle<TypstModAsset>,
    phantom: PhantomData<T>,
}

impl<T: Default> UiContext<T> {
    pub fn new(module: Handle<TypstModAsset>) -> Self {
        Self {
            module,
            ..default()
        }
    }

    pub fn module(&self) -> &Handle<TypstModAsset> {
        &self.module
    }

    pub fn on_hover(&mut self, label: TypLabel, func: impl FnOnce()) {}
    pub fn on_click(&mut self, label: TypLabel) {}
}

#[derive(Default)]
struct RootUi;

fn root_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiContext::<RootUi>::new(asset_server.load("game_ui.typ")));
}
