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
        .register_typst_func::<LabelFunc>()
        .register_typst_func::<ButtonFunc>()
        .register_typst_func::<PerfMetricsFunc>()
        .add_systems(Startup, setup)
        .add_systems(Update, (button_interaction, perf_metrics))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, VelloView));

    let handle =
        VelystSourceHandle(asset_server.load("typst/game_ui.typ"));

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::VMin(6.0)),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                VelystFuncBundle {
                    handle: handle.clone(),
                    func: LabelFunc::title("Title"),
                },
                VelystSize {
                    width: Val::Auto,
                    height: Val::Auto,
                },
                Node {
                    padding: UiRect::all(Val::Vh(4.0)),
                    margin: UiRect::vertical(Val::Vh(6.0)),
                    ..default()
                },
            ));
            builder.spawn((
                VelystFuncBundle {
                    handle: handle.clone(),
                    func: ButtonFunc::text("Start").with_fill(
                        viz::Color::from_u8(0, 255, 0, 255),
                    ),
                },
                VelystSize {
                    width: Val::Auto,
                    height: Val::Auto,
                },
                Node {
                    padding: UiRect::all(Val::Vh(4.0)),
                    ..default()
                },
                Button,
            ));

            builder.spawn((
                VelystFuncBundle {
                    handle,
                    func: PerfMetricsFunc::default(),
                },
                VelystSize {
                    width: Val::Auto,
                    height: Val::Auto,
                },
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    right: Val::Px(0.0),
                    ..default()
                },
            ));
        });
}

fn perf_metrics(
    time: Res<Time>,
    mut q_func: Query<&mut PerfMetricsFunc>,
) {
    let Ok(mut func) = q_func.get_single_mut() else {
        return;
    };

    func.fps = (1.0 / time.delta_secs_f64() * 100.0).round() / 100.0;
    func.elapsed_time =
        (time.elapsed_secs_f64() * 100.0).round() / 100.0;
}

fn button_interaction(
    mut q_buttons: Query<(&mut ButtonFunc, &Interaction)>,
) {
    for (mut func, interaction) in q_buttons.iter_mut() {
        func.hovered = *interaction == Interaction::Hovered;
    }
}

typst_func!(
    "perf_metrics",
    #[derive(Component, Default)]
    struct PerfMetricsFunc {},
    positional_args {
        fps: f64,
        elapsed_time: f64,
    },
);

typst_func!(
    "lbl",
    #[derive(Component, Default)]
    struct LabelFunc {},
    positional_args { body: Content },
    named_args {
        fill: viz::Color,
        size: Abs,
    }
);

impl LabelFunc {
    pub fn title(text: &str) -> Self {
        Self {
            body: elem::heading(elem::text(text).pack()).pack(),
            size: Some(Abs::pt(48.0)),
            ..default()
        }
    }

    // pub fn with_fill(mut self, fill: viz::Color) -> Self {
    //     self.fill = Some(fill);
    //     self
    // }
}

typst_func!(
    "button",
    #[derive(Component, Default)]
    struct ButtonFunc {},
    positional_args {
        body: Content,
        hovered: bool,
    },
    named_args {
        fill: viz::Color,
        size: Abs,
    }
);

impl ButtonFunc {
    pub fn text(text: &str) -> Self {
        Self {
            body: elem::heading(elem::text(text).pack()).pack(),
            ..default()
        }
    }

    pub fn with_fill(mut self, fill: viz::Color) -> Self {
        self.fill = Some(fill);
        self
    }
}
