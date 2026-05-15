use bevy::prelude::*;
use bevy_vello::prelude::*;
use velyst::imaging::Composite;
use velyst::imaging::kurbo::{BezPath, PathEl, Point};
use velyst::kanva::Kanva;
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
        .add_systems(
            PostUpdate,
            animate_logo.in_set(VelystSet::PostLayout),
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
            asset_server.load("typst/kanva_demo.typ"),
            MainFunc::default(),
        ),
        WorldScene::default(),
        // TODO: Using anchor has issues (with kanva only)!
        // .with_anchor(Vec2::splat(0.5)),
        VelystKanva::default(),
    ));
}

fn animate_logo(
    mut q_kanva: Query<&mut VelystKanva>,
    time: Res<Time>,
) {
    let Ok(mut kanva) = q_kanva.single_mut() else {
        return;
    };

    if kanva.is_empty() {
        return;
    }

    let t = time.elapsed_secs();

    // Frame pulses in opacity.
    if let Some(idx) = kanva.query_group("frame") {
        let t = (t * 1.5).sin().mul_add(0.5, 0.5);
        kanva.mod_group(idx).composite(Composite::new(
            default(),
            f32::lerp(0.3, 0.7, t),
        ));
    }

    // "k" glyph paths: perturb each control point's y by sin(x * freq + t).
    distort_group(&mut kanva, "letter_k", t, 40.0, 0.06);

    // "anva" glyph paths: same distortion, offset phase.
    distort_group(
        &mut kanva,
        "wordmark",
        t + std::f32::consts::PI,
        20.0,
        0.04,
    );

    // Accent line: rebuild as a sine wave each frame.
    wave_group(&mut kanva, "accent", t, 3.0, 20);
}

/// Perturbs every control point in a group's paths: dy = sin(x * x_freq + t) * amplitude.
fn distort_group(
    kanva: &mut Kanva,
    label: &str,
    t: f32,
    amplitude: f64,
    x_freq: f32,
) {
    let Some(group_idx) = kanva.query_group(label) else {
        return;
    };
    let Some(range) = kanva.get_group_path_range(group_idx) else {
        return;
    };

    let new_shapes: Vec<(usize, BezPath)> = range
        .filter_map(|path_idx| {
            let original = kanva.get_path(path_idx)?.path.clone();
            Some((
                path_idx,
                perturb_path(&original, t, amplitude, x_freq),
            ))
        })
        .collect();

    for (path_idx, new_path) in new_shapes {
        kanva.mod_path(path_idx).shape(new_path);
    }
}

/// Rebuilds a labeled path as a sine wave (expects simple MoveTo + LineTo).
fn wave_group(
    kanva: &mut Kanva,
    label: &str,
    t: f32,
    amplitude: f64,
    segments: usize,
) {
    let Some(path_idx) = kanva.query_path(label) else {
        return;
    };
    let Some(original) =
        kanva.get_path(path_idx).map(|p| p.path.clone())
    else {
        return;
    };
    kanva
        .mod_path(path_idx)
        .shape(wave_path(&original, t, amplitude, segments));
}

/// Shifts every control point's y by `sin(x * x_freq + t) * amplitude`.
fn perturb_path(
    original: &BezPath,
    t: f32,
    amplitude: f64,
    x_freq: f32,
) -> BezPath {
    let perturb = |p: Point| -> Point {
        let dy = ((p.x as f32 * x_freq + t * 3.0).sin() as f64)
            * amplitude;
        Point::new(p.x, p.y + dy)
    };

    original
        .elements()
        .iter()
        .map(|el| match el {
            PathEl::MoveTo(p) => PathEl::MoveTo(perturb(*p)),
            PathEl::LineTo(p) => PathEl::LineTo(perturb(*p)),
            PathEl::QuadTo(p1, p) => PathEl::QuadTo(*p1, perturb(*p)),
            PathEl::CurveTo(p1, p2, p) => {
                PathEl::CurveTo(*p1, *p2, perturb(*p))
            }
            PathEl::ClosePath => PathEl::ClosePath,
        })
        .collect()
}

/// Rebuilds a straight line path as a sine wave between its original endpoints.
fn wave_path(
    original: &BezPath,
    t: f32,
    amplitude: f64,
    segments: usize,
) -> BezPath {
    let els: Vec<PathEl> = original.elements().to_vec();

    let start = match els.first() {
        Some(PathEl::MoveTo(p)) => *p,
        _ => return original.clone(),
    };
    let end = match els.get(1) {
        Some(PathEl::LineTo(p)) => *p,
        _ => return original.clone(),
    };

    let mut path = BezPath::new();
    for i in 0..=segments {
        let nx = i as f64 / segments as f64;
        let x = start.x + nx * (end.x - start.x);
        let y = start.y
            + (nx as f32 * std::f32::consts::TAU * 2.0 + t * 4.0)
                .sin() as f64
                * amplitude;
        if i == 0 {
            path.move_to((x, y));
        } else {
            path.line_to((x, y));
        }
    }
    path
}

typst_func!(
    "main",
    #[derive(Default)]
    struct MainFunc {},
    positional_args {},
);
