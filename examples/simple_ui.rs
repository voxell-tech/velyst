use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};
use bevy_typst::{
    compiler::{world::TypstWorldMeta, TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};
use typst_element::prelude::*;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        // Custom plugins
        .add_plugins((TypstPlugin::default(), VelloPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, init_ui)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(asset_server.load::<TypstModAsset>("simple_ui.typ"));
}

fn init_ui(
    mut commands: Commands,
    q_simple_ui: Query<&Handle<TypstModAsset>>,
    typst_mod_assets: Res<Assets<TypstModAsset>>,
    compiler: Res<TypstCompiler>,
    mut initialized: Local<bool>,
) {
    if *initialized {
        return;
    }

    let world = compiler.world_meta();
    let Ok(simple_ui) = q_simple_ui.get_single() else {
        return;
    };

    if let Some(module) = typst_mod_assets.get(simple_ui).map(|asset| asset.module()) {
        let scope = module.scope();

        let red = scope.get_unchecked::<visualize::Color>("red");
        let orange = scope.get_unchecked::<visualize::Color>("orange");
        let yellow = scope.get_unchecked::<visualize::Color>("yellow");
        let green = scope.get_unchecked::<visualize::Color>("green");
        let blue = scope.get_unchecked::<visualize::Color>("blue");
        let purple = scope.get_unchecked::<visualize::Color>("purple");
        let base0 = scope.get_unchecked::<visualize::Color>("base0");
        let base1 = scope.get_unchecked::<visualize::Color>("base1");
        let base2 = scope.get_unchecked::<visualize::Color>("base2");
        let base3 = scope.get_unchecked::<visualize::Color>("base3");
        let base4 = scope.get_unchecked::<visualize::Color>("base4");
        let base5 = scope.get_unchecked::<visualize::Color>("base5");
        let base6 = scope.get_unchecked::<visualize::Color>("base6");
        let base7 = scope.get_unchecked::<visualize::Color>("base7");
        let base8 = scope.get_unchecked::<visualize::Color>("base8");

        let gradient_title = scope.get_unchecked::<foundations::Func>("gradient_title");
        let frame = scope.get_unchecked::<foundations::Func>("frame");
        let button = scope.get_unchecked::<foundations::Func>("button");
        let icon = scope.get_unchecked::<foundations::Func>("icon");

        // Create ui
        commands
            .spawn(EmptyNodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(40.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                let mut writer = SimpleWriter::new();
                writer.blank_page(|writer| {
                    writer.add_content(context(gradient_title.clone(), ["Typst"]));
                });

                let scene = typst_scene(writer, world);
                parent
                    .spawn(EmptyNodeBundle::from_typst(&scene))
                    .insert(vello_scene(scene));
            })
            .with_children(|parent| {
                let mut writer = SimpleWriter::new();
                writer.blank_page(|writer| {
                    writer.add_content(context(gradient_title, ["Typst"]));
                });

                let scene = typst_scene(writer, world);
                parent
                    .spawn(EmptyNodeBundle::from_typst(&scene))
                    .insert(vello_scene(scene));
            });

        *initialized = true;
    }
}

fn typst_scene(writer: impl DocWriter, world: &TypstWorldMeta) -> TypstScene {
    let document = world.compile_content(writer.pack()).unwrap();
    TypstScene::from_document(&document, Abs::zero()).unwrap()
}

fn vello_scene(scene: TypstScene) -> VelloSceneBundle {
    VelloSceneBundle {
        scene: VelloScene::from(scene.scene),
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..default()
    }
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

impl EmptyNodeBundle {
    pub fn from_typst(scene: &TypstScene) -> Self {
        Self {
            style: Style {
                width: Val::Px(scene.width),
                height: Val::Px(scene.height),
                ..default()
            },
            ..default()
        }
    }
}
