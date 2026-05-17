use bevy::prelude::*;
use bevy_vello::prelude::*;
use velyst::imaging::kurbo::{BezPath, PathEl, Point};
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
            animate_connections.in_set(VelystSet::PostLayout),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, VelloView));
    commands.spawn((
        VelystFunc::new(
            asset_server.load("typst/kanva_demo.typ"),
            MainFunc::default(),
        ),
        WorldScene::default(),
        // TODO: Using anchor has issues with glyph transforms
        // (with kanva only)!
        // .with_anchor(Vec2::splat(0.5)),
        VelystKanva::default(),
        ConnectionOriginals::default(),
        Transform::from_xyz(-250.0, 140.0, 0.0),
    ));
}

#[derive(Component, Default)]
struct ConnectionOriginals(Vec<(Point, Point)>);

fn animate_connections(
    mut q: Query<(&mut VelystKanva, &mut ConnectionOriginals)>,
    time: Res<Time>,
) {
    let Ok((mut kanva, mut originals)) = q.single_mut() else {
        return;
    };

    if kanva.is_empty() {
        return;
    }

    let Some(group_idx) = kanva.query_group("connections") else {
        return;
    };
    let Some(range) = kanva.get_group_path_range(group_idx) else {
        return;
    };
    let indices = range.collect::<Box<_>>();

    // Cache original endpoints on the first frame.
    if originals.0.is_empty() {
        for &idx in &indices {
            let Some(entry) = kanva.get_path(idx) else {
                continue;
            };
            let els = entry.path.elements().to_vec();
            if let (
                Some(PathEl::MoveTo(start)),
                Some(PathEl::LineTo(end)),
            ) = (els.first(), els.get(1))
            {
                originals.0.push((*start, *end));
            }
        }
        return;
    }

    let stagger = 0.05;
    let duration = 0.3;
    let n = indices.len() as f32;
    let cycle = n * stagger + duration + 0.5;
    let t = time.elapsed_secs() % cycle;

    for (i, (&path_idx, &(start, end))) in
        indices.iter().zip(originals.0.iter()).enumerate()
    {
        let progress = ((t - i as f32 * stagger) / duration)
            .clamp(0.0, 1.0) as f64;

        let cur_end = Point::new(
            start.x + progress * (end.x - start.x),
            start.y + progress * (end.y - start.y),
        );

        let mut path = BezPath::new();
        path.move_to(start);
        path.line_to(cur_end);
        kanva.mod_path(path_idx).shape(path);
    }
}

typst_func!(
    "main",
    #[derive(Default)]
    struct MainFunc {},
    positional_args {},
);
