use bevy::{prelude::*, window::PrimaryWindow};
use bevy_typst::{
    compiler::{TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{VelloPlugin, VelloScene, VelloSceneBundle};
use typst::{
    foundations::{Content, NativeElement, SequenceElem, Smart, Style},
    layout::{self, Abs},
};
use typst_element::{align, block, heading, page, scale, sequence, text};

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        // Custom plugins
        .add_plugins((TypstPlugin::default(), VelloPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, ui_update)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    // commands.spawn((VelloAssetBundle {
    //     vector: asset_server.load("simple_ui.typ"),
    //     ..default()
    // },));

    commands.spawn(VelloSceneBundle::default());
}

fn ui_update(
    mut q_scene: Query<(&mut VelloScene, &mut Transform)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    world: Res<TypstCompiler>,
    time: Res<Time>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let width = window.width() as f64;
    let height = window.height() as f64;

    println!("width: {width}, height: {height}");

    let Ok((mut scene, mut transform)) = q_scene.get_single_mut() else {
        return;
    };

    fn length(pt: f64) -> layout::Length {
        layout::Length::from(layout::Abs::pt(pt))
    }

    let content = align(
        page(
            scale(
                block()
                    .with_width(Smart::Custom(layout::Rel::new(
                        layout::Ratio::zero(),
                        length(width),
                    )))
                    .with_height(Smart::Custom(layout::Rel::new(
                        layout::Ratio::zero(),
                        length(height),
                    )))
                    .with_body(Some(
                        sequence!(
                            heading(text(time.elapsed_seconds().to_string())),
                            text((1.0 / time.delta_seconds()).to_string())
                        )
                        .pack(),
                    )),
            )
            .with_x(layout::Ratio::new(0.75))
            .with_y(layout::Ratio::new(0.75))
            .with_reflow(true),
        )
        .with_width(Smart::Auto)
        .with_height(Smart::Auto)
        .with_margin(layout::Margin::splat(Some(Smart::Custom(
            layout::Rel::zero(),
        )))),
    )
    .with_alignment(layout::Alignment::Both(
        layout::HAlignment::Center,
        layout::VAlignment::Horizon,
    ))
    .pack();

    let mut document = world.compile_content(content).unwrap();
    let frame = std::mem::take(&mut document.pages[0].frame);
    document.pages[0].frame = frame.mark_box();
    let typst_scene = TypstScene::from_document(&document, Abs::zero()).unwrap();

    println!(
        "scene: width: {}, height: {}",
        typst_scene.width, typst_scene.height
    );

    *transform = Transform::from_xyz(-typst_scene.width * 0.5, typst_scene.height * 0.5, 0.0);
    *scene = typst_scene.as_component();
}

#[derive(Default)]
pub struct TypstBuidler(pub Vec<Content>);

impl TypstBuidler {
    pub fn add_content(&mut self, content: Content) -> &mut Content {
        self.0.push(content);
        self.0.last_mut().unwrap()
    }

    pub fn pack(self) -> Content {
        SequenceElem::new(self.0).pack()
    }
}

impl TypstBuidler {
    pub fn new(children: Vec<Content>) -> Self {
        Self(children)
    }

    // pub fn page(&mut self, builder_fn: impl Fn(&mut TypstBuidler)) -> ContentMut {
    //     let mut builder = Self::default();
    //     builder_fn(&mut builder);

    //     let content = PageElem::new(builder.pack()).pack();
    //     ContentMut(self.add_content(content))
    // }

    // pub fn text(&mut self, text: impl Into<EcoString>) -> ContentMut {
    //     let content = TextElem::new(text.into()).pack();
    //     ContentMut(self.add_content(content))
    // }

    // // pub fn quote(&mut self, builder_fn: impl Fn(&mut SequenceBuidler)) -> ContentMut {
    // //     let mut builder = Self::default();
    // //     builder_fn(&mut builder);

    // //     let content = QuoteElem::new(builder.pack()).pack();
    // //     ContentMut(self.add_content(content))
    // // }

    // pub fn heading(&mut self, builder_fn: impl Fn(&mut TypstBuidler)) -> ContentMut {
    //     let mut builder = Self::default();
    //     builder_fn(&mut builder);

    //     let content = HeadingElem::new(builder.pack()).pack();
    //     ContentMut(self.add_content(content))
    // }

    // // pub fn boxed(&mut self, builder_fn: impl Fn(&mut SequenceBuidler)) -> ContentMut {
    // //     let mut builder = Self::default();
    // //     builder_fn(&mut builder);

    // //     let mut content = BoxElem::new().with_body();
    // //     let content = BoxElem::new(builder.pack()).pack();
    // //     ContentMut(self.add_content(content))
    // // }

    // pub fn par(&mut self, builder_fn: impl Fn(&mut TypstBuidler)) -> ContentMut {
    //     let mut builder = Self::default();
    //     builder_fn(&mut builder);

    //     let content = ParElem::new(builder.0).pack();
    //     ContentMut(self.add_content(content))
    // }

    // pub fn linebreak(&mut self) -> ContentMut {
    //     let content = LinebreakElem::new().pack();
    //     ContentMut(self.add_content(content))
    // }

    // pub fn parbreak(&mut self) -> ContentMut {
    //     let content = ParbreakElem::new().pack();
    //     ContentMut(self.add_content(content))
    // }

    // pub fn bullet_list(&mut self, builder_fn: impl Fn(&mut TypstBuidler)) -> ContentMut {
    //     let mut builder = Self::default();
    //     builder_fn(&mut builder);

    //     let content = ListElem::new(
    //         builder
    //             .0
    //             .drain(..)
    //             .map(|c| Packed::new(ListItem::new(c)))
    //             .collect(),
    //     )
    //     .pack();
    //     ContentMut(self.add_content(content))
    // }

    // pub fn numbered_list(&mut self, builder_fn: impl Fn(&mut TypstBuidler)) -> ContentMut {
    //     let mut builder = Self::default();
    //     builder_fn(&mut builder);

    //     let content = EnumElem::new(
    //         builder
    //             .0
    //             .drain(..)
    //             .map(|c| Packed::new(EnumItem::new(c)))
    //             .collect(),
    //     )
    //     .pack();
    //     ContentMut(self.add_content(content))
    // }
}

pub struct ContentMut<'a>(&'a mut Content);

impl<'a> ContentMut<'a> {
    pub fn style(self, style: impl Into<Style>) -> Self {
        let content_value = std::mem::take(self.0);
        *self.0 = content_value.styled(style);
        self
    }
}
