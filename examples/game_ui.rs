use std::marker::PhantomData;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_typst::{compiler::TypstCompiler, TypstPlugin};
use bevy_vello::{prelude::*, VelloPlugin};
use typst::World;
use typst_element::prelude::*;
use typst_vello::TypstScene;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(TypstPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, perf_test)
        .run();
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..default()
    });
}

pub fn perf_test(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_scene: Query<&mut VelloScene>,
    compiler: Res<TypstCompiler>,
    time: Res<Time>,
) {
    let Ok(window_size) = q_window.get_single().map(|w| w.size()) else {
        return;
    };
    let Ok(mut scene) = q_scene.get_single_mut() else {
        return;
    };

    let linear_gradient = std::sync::Arc::new(viz::LinearGradient {
        stops: vec![
            (viz::Color::BLUE, Ratio::zero()),
            (viz::Color::RED, Ratio::one()),
        ],
        angle: layout::Angle::zero(),
        space: viz::ColorSpace::Oklab,
        relative: Smart::Custom(viz::RelativeTo::Self_),
        anti_alias: false,
    });

    let content = boxed()
        .with_body(Some(Content::sequence([
            heading(
                text("Performance Metrics:")
                    .pack()
                    .labelled(TypLabel::new("title")),
            )
            .pack(),
            text(format!("FPS: {}", 1.0 / time.delta_seconds())).pack(),
            linebreak().pack(),
            text(format!("Elapsed Time: {}", time.elapsed_seconds())).pack(),
            linebreak().pack(),
            // Boxes
            boxed()
                .with_body(Some(
                    rotate(
                        rect()
                            .with_width(Abs::pt(50.0).smart_rel())
                            .with_height(Abs::pt(50.0).smart_rel())
                            .with_fill(Some(gradient(viz::Gradient::Linear(linear_gradient)))),
                    )
                    .with_angle(layout::Angle::rad(
                        time.elapsed_seconds_f64() * std::f64::consts::TAU,
                    ))
                    .pack(),
                ))
                .pack()
                .labelled(TypLabel::new("rotated-box"))
                .repeat(100),
        ])))
        // .with_width(layout::Sizing::Rel(Abs::pt(window_size.x as f64).rel()))
        // .with_height(Abs::pt(window_size.y as f64).smart_rel())
        .with_inset(layout::Sides::splat(Some(Abs::pt(40.0).rel())))
        .pack()
        .styled(text::TextElem::set_size(text::TextSize(
            Abs::pt(24.0).length(),
        )))
        .styled(text::TextElem::set_fill(solid(viz::Luma::new(0.8, 1.0))));

    // Will not work
    // if let Some(mut title) =
    //     content.query_first(foundations::Selector::Label(TypLabel::new("title")))
    // {
    //     let title = title.to_packed_mut::<text::TextElem>().unwrap();
    //     title.push_text(format!("Changed Title: {}", time.delta_seconds()).into());
    // }

    let frame = compiler
        .scoped_engine(|engine| {
            let locator = typst::introspection::Locator::root();
            let styles = foundations::StyleChain::new(&compiler.library().styles);

            typst::layout::layout_frame(
                engine,
                &content,
                locator,
                styles,
                layout::Region::new(
                    layout::Axes::new(Abs::pt(window_size.x as f64), Abs::pt(window_size.y as f64)),
                    layout::Axes::new(true, true),
                ),
            )
        })
        .unwrap_or_default();

    let mut typst_scene = TypstScene::from_frame(&frame);
    typst_scene
        .query(TypLabel::new("rotated-box"))
        .iter()
        .for_each(|i| {
            let group = typst_scene.get_group_mut(*i);
            let mut scale_factor = (time.elapsed_seconds_f64() * 4.0).sin() * 0.5 + 0.5;
            scale_factor *= 0.4;
            let scale = 1.0 + scale_factor;
            group.transform *=
                kurbo::Affine::scale(scale).with_translation(-group.size * scale_factor * 0.5);
        });

    *scene = VelloScene::from(typst_scene.render());
}

// pub struct TypstUi(Vec<Content>);

// impl DocWriter for TypstUi {
//     fn contents(&self) -> &Vec<Content> {
//         &self.0
//     }

//     fn contents_mut(&mut self) -> &mut Vec<Content> {
//         &mut self.0
//     }

//     fn take_contents(self) -> Vec<Content> {
//         self.0
//     }
// }

#[derive(Resource)]
pub struct UiContext<T> {
    typst_scene: TypstScene,
    boxed: Packed<layout::BoxElem>,
    phantom: PhantomData<T>,
}

impl<T> UiContext<T> {
    pub fn on_hover(label: TypLabel) {}
    pub fn on_click(label: TypLabel) {}
}

struct RootUi;

fn root_ui(ctx: ResMut<UiContext<RootUi>>) {}
