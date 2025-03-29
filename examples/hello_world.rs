use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_vello::prelude::*;
use typst_element::prelude::*;
use velyst::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_vello::VelloPlugin::default(),
            velyst::VelystPlugin,
        ))
        .register_typst_asset::<HelloWorld>()
        .compile_typst_func::<HelloWorld, MainFunc>()
        .render_typst_func::<MainFunc>()
        .add_systems(Startup, setup)
        .init_resource::<MainFunc>()
        .add_systems(Update, main_func)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: Color::BLACK.into(),
            ..default()
        },
        VelloView,
    ));
}

fn main_func(
    q_window: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut main_func: ResMut<MainFunc>,
    time: Res<Time>,
) {
    if let Ok(window) = q_window.get_single() {
        main_func.width = window.width() as f64;
        main_func.height = window.height() as f64;
    };

    main_func.animate = time.elapsed_secs_f64();
}

#[derive(Resource, Default)]
// #[typst_func(name = "main")]
struct MainFunc {
    width: f64,
    height: f64,
    // #[typst_func(named)]
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

#[derive(TypstPath)]
#[typst_path = "typst/hello_world.typ"]
struct HelloWorld;
