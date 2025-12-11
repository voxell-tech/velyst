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
        .add_systems(Update, main_func)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: Color::BLACK.into(),
            ..default()
        },
        VelloView,
    ));

    // let debug_bg = BackgroundColor(Srgba::RED.with_alpha(0.2).into());
    let debug_bg = BackgroundColor::DEFAULT;

    commands.spawn((
        debug_bg,
        VelystFuncBundle {
            handle: VelystSourceHandle(
                asset_server.load("typst/hello_world.typ"),
            ),
            func: MainFunc::default(),
        },
        Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
    ));
}

fn main_func(
    mut func: Query<&mut MainFunc>,
    time: Res<Time>,
) -> Result {
    let mut func = func.single_mut()?;
    func.animate = time.elapsed_secs_f64();

    Ok(())
}

typst_func!(
    "main",
    #[derive(Component, Default)]
    struct MainFunc {},
    positional_args { animate: f64 },
);
