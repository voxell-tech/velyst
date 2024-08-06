use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};
use bevy_typst::{
    compiler::{TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};
use typst::visualize;
use typst_element::{elem::ContentExt, prelude::*, UnitExt};

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

    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..default()
    });
}

fn ui_update(
    mut q_scene: Query<&mut VelloScene>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    world: Res<TypstCompiler>,
    time: Res<Time>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let width = window.width() as f64;
    let height = window.height() as f64;

    if width <= 0.0 || height <= 0.0 {
        return;
    }

    let Ok(mut scene) = q_scene.get_single_mut() else {
        return;
    };

    let gradient = visualize::Gradient::Linear(std::sync::Arc::new(visualize::LinearGradient {
        stops: vec![],
        angle: layout::Angle::zero(),
        space: visualize::ColorSpace::Srgb,
        relative: Smart::Auto,
        anti_alias: true,
    }));

    let mut writer = SimpleWriter::new();

    writer.blank_page(|writer| {
        writer.add_content(
            sequence!(
                heading(text(time.elapsed_seconds().to_string())),
                text((1.0 / time.delta_seconds()).to_string())
            )
            .align(layout::Alignment::Both(
                layout::HAlignment::Center,
                layout::VAlignment::Horizon,
            )),
        );
    });

    let content = writer
        .pack()
        .styled(text::TextElem::set_fill(visualize::Paint::Solid(
            visualize::Color::WHITE,
        )))
        .styled(text::TextElem::set_size(text::TextSize(
            Abs::pt(24.0).length(),
        )));

    let document = world.compile_content(content).unwrap();
    let typst_scene = TypstScene::from_document(&document, Abs::zero()).unwrap();

    *scene = typst_scene.as_component();
}

// pub struct UiWriter<'w, 's>(Vec<Content>, Commands<'w, 's>);
pub struct UiWriter(Vec<Content>);

impl DocWriter for UiWriter {
    fn contents(&self) -> &Vec<Content> {
        &self.0
    }

    fn contents_mut(&mut self) -> &mut Vec<Content> {
        &mut self.0
    }

    fn take_contents(self) -> Vec<Content> {
        self.0
    }
}

impl UiWriter {
    // pub fn commands(&mut self) -> Commands {
    //     self.1.reborrow()
    // }
}

#[derive(Bundle, Clone, Debug, Default)]
pub struct EmptyNodeBundle {
    /// Describes the logical size of the node
    pub node: Node,
    /// Styles which control the layout (size and position) of the node and its children
    /// In some cases these styles also affect how the node drawn/painted.
    pub style: Style,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    ///
    /// This component is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub transform: Transform,
    /// The global transform of the node
    ///
    /// This component is automatically updated by the [`TransformPropagate`](`bevy_transform::TransformSystem::TransformPropagate`) systems.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub z_index: ZIndex,
}
