use std::str::FromStr;

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

    let handle = asset_server.load("typst/game_ui.typ");

    // Colors.
    const RED: &str = "#FF6188";
    const GREEN: &str = "#A9DC76";
    const PURPLE: &str = "#AB9DF2";

    let green = viz::Color::from_str(GREEN).unwrap();
    let purple = viz::Color::from_str(PURPLE).unwrap();
    let red = viz::Color::from_str(RED).unwrap();

    // let debug_bg = BackgroundColor(Srgba::RED.with_alpha(0.2).into());
    let debug_bg = BackgroundColor::DEFAULT;

    commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(vmin(6.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|builder| {
            // Title.
            builder.spawn((
                debug_bg,
                VelystFunc::new(
                    handle.clone(),
                    LabelFunc::title("Title"),
                ),
                UiScene,
                Node {
                    width: auto(),
                    height: auto(),
                    margin: UiRect::vertical(vh(10.0)),
                    ..default()
                },
            ));

            // Buttons.
            let mut create_button = |func: ButtonFunc| {
                builder.spawn((
                    debug_bg,
                    VelystFunc::new(handle.clone(), func),
                    UiScene,
                    Node {
                        width: auto(),
                        height: auto(),
                        margin: UiRect::all(vh(2.0)),
                        ..default()
                    },
                    Button,
                ));
            };

            create_button(ButtonFunc::text("Start").with_fill(green));
            create_button(
                ButtonFunc::text("Settings").with_fill(purple),
            );
            create_button(ButtonFunc::text("Exit").with_fill(red));

            builder.spawn((
                debug_bg,
                VelystFunc::new(handle, PerfMetricsFunc::default()),
                UiScene,
                Node {
                    width: auto(),
                    height: auto(),
                    position_type: PositionType::Absolute,
                    bottom: px(0.0),
                    right: px(0.0),
                    ..default()
                },
            ));
        });
}

fn perf_metrics(
    time: Res<Time>,
    mut q_func: Query<&mut VelystFunc<PerfMetricsFunc>>,
) -> Result {
    let mut func = q_func.single_mut()?;

    func.data.fps =
        (1.0 / time.delta_secs_f64() * 100.0).round() / 100.0;
    func.data.elapsed_time =
        (time.elapsed_secs_f64() * 100.0).round() / 100.0;

    Ok(())
}

fn button_interaction(
    mut q_buttons: Query<
        (&mut VelystFunc<ButtonFunc>, &Interaction),
        Changed<Interaction>,
    >,
) {
    for (mut func, interaction) in q_buttons.iter_mut() {
        func.data.interaction_state = match interaction {
            Interaction::Pressed => 2,
            Interaction::Hovered => 1,
            Interaction::None => 0,
        };
    }
}

typst_func!(
    "perf_metrics",
    #[derive(Default)]
    struct PerfMetricsFunc {},
    positional_args {
        fps: f64,
        elapsed_time: f64,
    },
);

typst_func!(
    "lbl",
    #[derive(Default)]
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
}

typst_func!(
    "button",
    #[derive(Default)]
    struct ButtonFunc {},
    positional_args {
        body: Content,
        interaction_state: u8,
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
