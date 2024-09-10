use bevy::{prelude::*, window::PrimaryWindow};
use bevy_typst::{prelude::*, typst_element::prelude::*, VelystPlugin};
use bevy_vello::VelloPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(VelystPlugin::default())
        .register_typst_asset::<GameUi>()
        .compile_typst_func::<GameUi, MainFunc>()
        .compile_typst_func::<GameUi, PerfMetricsFunc>()
        .render_typst_func::<MainFunc>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            main_func.run_if(resource_exists::<TypstContent<PerfMetricsFunc>>),
        )
        .add_systems(Update, perf_metrics)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

struct GameUi;

impl TypstPath for GameUi {
    fn path() -> &'static str {
        "game_ui.typ"
    }
}

fn main_func(
    mut commands: Commands,
    q_window: Query<Ref<Window>, With<PrimaryWindow>>,
    q_interactions: Query<(&Interaction, &TypstLabel)>,
    perf_metrics: Res<TypstContent<PerfMetricsFunc>>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let mut btn_highlight = String::new();

    for (interaction, label) in q_interactions.iter() {
        if *interaction == Interaction::Hovered {
            btn_highlight = label.as_str().to_owned();

            break;
        }
    }

    if window.is_changed() || perf_metrics.is_changed() {
        commands.insert_resource(MainFunc {
            width: window.width() as f64,
            height: window.height() as f64,
            perf_metrics: perf_metrics.clone(),
            btn_highlight,
        });
    }
}

fn perf_metrics(mut commands: Commands, time: Res<Time>) {
    let fps = (1.0 / time.delta_seconds_f64() * 100.0).round() / 100.0;
    let elapsed_time = (time.elapsed_seconds_f64() * 100.0).round() / 100.0;

    commands.insert_resource(PerfMetricsFunc { fps, elapsed_time });
}

#[derive(Resource)]
pub struct MainFunc {
    width: f64,
    height: f64,
    perf_metrics: Content,
    btn_highlight: String,
}

impl TypstFunc for MainFunc {
    fn func_name(&self) -> &str {
        "main"
    }

    fn content(&self, func: foundations::Func) -> Content {
        context(func, |args| {
            args.push(self.width);
            args.push(self.height);
            args.push(self.perf_metrics.clone());
            args.push_named("btn_highlight", self.btn_highlight.clone());
        })
        .pack()
    }
}

#[derive(Resource)]
pub struct PerfMetricsFunc {
    fps: f64,
    elapsed_time: f64,
}

impl TypstFunc for PerfMetricsFunc {
    fn func_name(&self) -> &str {
        "perf_metrics"
    }

    fn content(&self, func: foundations::Func) -> Content {
        context(func, |args| {
            args.push(self.fps);
            args.push(self.elapsed_time);
        })
        .pack()
    }
}
