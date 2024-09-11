use bevy::{prelude::*, window::PrimaryWindow};
use bevy_vello::VelloPlugin;
use velyst::{prelude::*, typst_element::prelude::*, VelystPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(VelystPlugin::default())
        .register_typst_asset::<GameUi>()
        .compile_typst_func::<GameUi, MainFunc>()
        .compile_typst_func::<GameUi, PerfMetricsFunc>()
        .render_typst_func::<MainFunc>()
        .add_systems(Startup, setup)
        .init_resource::<MainFunc>()
        .init_resource::<PerfMetricsFunc>()
        .add_systems(
            Update,
            (main_func_window, main_func_interactions, main_func_metrics),
        )
        .add_systems(Update, perf_metrics)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main_func_window(
    q_window: Query<Ref<Window>, (With<PrimaryWindow>, Changed<Window>)>,
    mut main_func: ResMut<MainFunc>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    main_func.width = window.width() as f64;
    main_func.height = window.height() as f64;
}

fn main_func_metrics(
    perf_metrics: Res<TypstContent<PerfMetricsFunc>>,
    mut main_func: ResMut<MainFunc>,
) {
    if perf_metrics.is_changed() {
        main_func.perf_metrics = perf_metrics.clone();
    }
}

fn main_func_interactions(
    q_interactions: Query<(&Interaction, &TypstLabel), Changed<Interaction>>,
    mut main_func: ResMut<MainFunc>,
    time: Res<Time>,
) {
    for (interaction, label) in q_interactions.iter() {
        if *interaction == Interaction::Hovered {
            main_func.btn_highlight = label.as_str().to_owned();
            main_func.animate = 0.0;
        } else {
            main_func.btn_highlight.clear();
        }
    }

    // Clamp below 1.0
    const SPEED: f64 = 8.0;
    main_func.animate = f64::min(main_func.animate + time.delta_seconds_f64() * SPEED, 1.0);
}

fn perf_metrics(time: Res<Time>, mut perf_metrics: ResMut<PerfMetricsFunc>) {
    let fps = (1.0 / time.delta_seconds_f64() * 100.0).round() / 100.0;
    let elapsed_time = (time.elapsed_seconds_f64() * 100.0).round() / 100.0;

    perf_metrics.fps = fps;
    perf_metrics.elapsed_time = elapsed_time;
}

struct GameUi;

impl TypstPath for GameUi {
    fn path() -> &'static str {
        "typst/game_ui.typ"
    }
}

#[derive(Resource, Default)]
pub struct MainFunc {
    width: f64,
    height: f64,
    perf_metrics: Content,
    btn_highlight: String,
    animate: f64,
}

impl TypstFunc for MainFunc {
    fn func_name(&self) -> &str {
        "main"
    }

    fn content(&self, func: foundations::Func) -> Content {
        elem::context(func, |args| {
            args.push(self.width);
            args.push(self.height);
            args.push(self.perf_metrics.clone());
            args.push_named("btn_highlight", self.btn_highlight.clone());
            args.push_named("animate", self.animate);
        })
        .pack()
    }
}

#[derive(Resource, Default)]
pub struct PerfMetricsFunc {
    fps: f64,
    elapsed_time: f64,
}

impl TypstFunc for PerfMetricsFunc {
    fn func_name(&self) -> &str {
        "perf_metrics"
    }

    fn content(&self, func: foundations::Func) -> Content {
        elem::context(func, |args| {
            args.push(self.fps);
            args.push(self.elapsed_time);
        })
        .pack()
    }
}
