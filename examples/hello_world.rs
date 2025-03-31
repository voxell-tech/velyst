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

    let handle = VelystSourceHandle(asset_server.load("typst/hello_world.typ"));
    commands.spawn((
        VelystFuncBundle {
            handle,
            func: MainFunc { animate: 0.0 },
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
}

fn main_func(mut func: Query<&mut MainFunc>, time: Res<Time>) {
    let mut func = func.single_mut();
    func.animate = time.elapsed_secs_f64();
}

typst_func!(
    #[derive(Component)]
    struct MainFunc {
        animate: f64,
    },
    "main"
);
