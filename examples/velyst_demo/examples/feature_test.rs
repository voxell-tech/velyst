use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_vello::prelude::*;
use velyst::prelude::*;

const ZOOM_LINE_RATE: f32 = 0.3;
const ZOOM_PIXEL_RATE: f32 = 0.005;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_vello::VelloPlugin::default(),
            velyst::VelystPlugin,
        ))
        .register_typst_func::<FeatureTestFunc>()
        .add_systems(Startup, setup)
        .add_systems(Update, pan_zoom_camera)
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
            asset_server.load("typst/feature_test.typ"),
            FeatureTestFunc::default(),
        ),
        WorldScene::default().with_anchor(Vec2::splat(0.5)),
    ));
}

fn pan_zoom_camera(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_q: Query<
        (&Camera, &mut Transform, &mut Projection),
        With<Camera2d>,
    >,
    mut scroll: MessageReader<MouseWheel>,
    mut last_pos: Local<Option<Vec2>>,
) {
    let Ok(window) = primary_window.single() else {
        return;
    };

    let current_pos = match window.cursor_position() {
        Some(p) => vec2(p.x, -p.y),
        None => return,
    };
    let delta_pixels = current_pos - last_pos.unwrap_or(current_pos);
    *last_pos = Some(current_pos);

    let Ok((camera, mut transform, mut projection)) =
        camera_q.single_mut()
    else {
        return;
    };

    let Projection::Orthographic(ref mut ortho) = *projection else {
        return;
    };

    for ev in scroll.read() {
        let delta = match ev.unit {
            MouseScrollUnit::Line => ev.y * ZOOM_LINE_RATE,
            MouseScrollUnit::Pixel => ev.y * ZOOM_PIXEL_RATE,
        };
        ortho.scale = (ortho.scale * (1.0 - delta)).clamp(0.05, 10.0);
    }

    if mouse_buttons.pressed(MouseButton::Left)
        && delta_pixels != Vec2::ZERO
    {
        let viewport_size =
            camera.logical_viewport_size().unwrap_or(window.size());
        let world_units_per_pixel = ortho.area.size() / viewport_size;
        let delta = delta_pixels * world_units_per_pixel;
        transform.translation.x -= delta.x;
        transform.translation.y -= delta.y;
    }
}

typst_func!(
    "feature_test",
    #[derive(Default)]
    struct FeatureTestFunc {},
);
