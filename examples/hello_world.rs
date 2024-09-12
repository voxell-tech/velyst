use bevy::{prelude::*, window::PrimaryWindow};
use bevy_vello::VelloPlugin;
use velyst::{prelude::*, typst_element::prelude::*, VelystPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(VelystPlugin::default())
        .register_typst_asset::<HelloWorld>()
        .compile_typst_func::<HelloWorld, MainFunc>()
        .render_typst_func::<MainFunc>()
        .add_systems(Startup, setup)
        .init_resource::<MainFunc>()
        .add_systems(Update, main_func)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: Color::BLACK.into(),
            ..default()
        },
        ..default()
    });
}

fn main_func(
    q_window: Query<Ref<Window>, (With<PrimaryWindow>, Changed<Window>)>,
    mut main_func: ResMut<MainFunc>,
    time: Res<Time>,
) {
    if let Ok(window) = q_window.get_single() {
        main_func.width = window.width() as f64;
        main_func.height = window.height() as f64;
    };

    main_func.animate = time.elapsed_seconds_f64();
}

struct HelloWorld;

impl TypstPath for HelloWorld {
    fn path() -> &'static str {
        "typst/hello_world.typ"
    }
}

#[derive(Resource, Default)]
struct MainFunc {
    width: f64,
    height: f64,
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
            args.push_named("animate", self.animate);
        })
        .pack()
    }
}
