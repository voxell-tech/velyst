use bevy::{prelude::*, ui::FocusPolicy};
use bevy_typst::{
    compiler::{world::TypstWorldMeta, TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};
use typst_element::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(TypstPlugin::default())
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

        let purple = scope.get_unchecked_color("purple");
        let gradient_title = scope.get_unchecked_func("gradient_title");
        let menu_item = scope.get_unchecked_func("menu_item");

        // Create ui
        commands
            .spawn(EmptyNodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(60.0)),
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
                parent.spawn(EmptyNodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                });

                parent
                    .spawn(EmptyNodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            justify_items: JustifyItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        let create_menu = |title: Content, body: Content| -> TypstScene {
                            let mut writer = SimpleWriter::new();
                            writer.blank_page(|writer| {
                                writer.add_content(context(menu_item.clone(), [title, body]));
                            });
                            typst_scene(writer, world)
                        };

                        // First item
                        let scene0 = create_menu(
                            heading(text("Model"))
                                .pack()
                                .styled(text::TextElem::set_fill(solid(purple))),
                            text("Document structuring.").pack(),
                        );
                        parent
                            .spawn(EmptyNodeBundle::from_typst(&scene0))
                            .insert(vello_scene(scene0));

                        // Second item
                        let scene1 = create_menu(
                            heading(text("Layout"))
                                .pack()
                                .styled(text::TextElem::set_fill(solid(purple))),
                            text("Arranging elements on the page in different ways.").pack(),
                        );
                        parent
                            .spawn(EmptyNodeBundle::from_typst(&scene1))
                            .insert(vello_scene(scene1));

                        // Third item
                        let scene2 = create_menu(
                            heading(text("Visualize"))
                                .pack()
                                .styled(text::TextElem::set_fill(solid(purple))),
                            text("Drawing and data visualization.").pack(),
                        );
                        parent
                            .spawn(EmptyNodeBundle::from_typst(&scene2))
                            .insert(vello_scene(scene2));
                    });

                parent.spawn(EmptyNodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                });
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
