use bevy::prelude::*;
use bevy_vello::prelude::*;
use velyst::imaging::Composite;
use velyst::kanva::prelude::GroupModifier;
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
        .add_systems(
            PostUpdate,
            animate_kanva.in_set(VelystSet::PostLayout),
        )
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

    commands.spawn((
        VelystFunc::new(
            asset_server.load("typst/hello_world.typ"),
            MainFunc::default(),
        ),
        UiScene,
        VelystKanva::default(),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
}

/// Fade the first group in and out using a [`GroupModifier`] composite alpha.
fn animate_kanva(
    mut q_kanva: Query<&mut VelystKanva>,
    time: Res<Time>,
) {
    let Ok(mut kanva) = q_kanva.single_mut() else {
        return;
    };

    let group_idx =
        kanva.0.query_group("wave").expect("should have wave group");

    let alpha =
        (time.elapsed_secs().sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    kanva.0.set_group_mod(
        group_idx,
        GroupModifier {
            composite: Some(Composite::new(default(), alpha)),
            ..default()
        },
    );
}

fn main_func(
    mut q_func: Query<&mut VelystFunc<MainFunc>>,
    time: Res<Time>,
) -> Result {
    let mut func = q_func.single_mut()?;
    func.data.animate = time.elapsed_secs_f64();

    Ok(())
}

typst_func!(
    "main",
    #[derive(Default)]
    struct MainFunc {},
    positional_args { animate: f64 },
);
