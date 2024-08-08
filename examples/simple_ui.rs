use bevy::{prelude::*, ui::FocusPolicy, window::PrimaryWindow};
use bevy_typst::{
    compiler::{TypstCompiler, TypstScene},
    prelude::*,
};
use bevy_vello::{prelude::*, VelloPlugin};
use typst::visualize;
use typst_element::{prelude::*, UnitExt};

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        // Custom plugins
        .add_plugins((TypstPlugin::default(), VelloPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_template.run_if(not(resource_exists::<Template>)),
                ui_update.run_if(resource_exists::<Template>),
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(asset_server.load::<TypstModAsset>("simple_ui.typ"));

    // let value = module.scope().get("red").unwrap();
    // let color = value.clone().cast::<typst::visualize::Color>();
    // println!("{color:?}");

    commands.spawn(VelloSceneBundle {
        coordinate_space: CoordinateSpace::ScreenSpace,
        ..default()
    });
}

#[derive(Resource)]
struct Template {
    pub frame: typst::foundations::Func,
    pub button: typst::foundations::Func,
    pub icon: typst::foundations::Func,
}

#[derive(Resource)]
struct ColorTemplate {
    pub red: typst::visualize::Color,
    pub orange: typst::visualize::Color,
    pub yellow: typst::visualize::Color,
    pub green: typst::visualize::Color,
    pub blue: typst::visualize::Color,
    pub purple: typst::visualize::Color,
    pub base0: typst::visualize::Color,
    pub base1: typst::visualize::Color,
    pub base2: typst::visualize::Color,
    pub base3: typst::visualize::Color,
    pub base4: typst::visualize::Color,
    pub base5: typst::visualize::Color,
    pub base6: typst::visualize::Color,
    pub base7: typst::visualize::Color,
    pub base8: typst::visualize::Color,
}

pub struct Test;

fn init_template(
    mut commands: Commands,
    q_simple_ui: Query<&Handle<TypstModAsset>>,
    typst_mod_assets: Res<Assets<TypstModAsset>>,
) {
    let Ok(simple_ui) = q_simple_ui.get_single() else {
        return;
    };

    if let Some(module) = typst_mod_assets.get(simple_ui).map(|asset| asset.module()) {
        let scope = module.scope();

        let red = scope.get("red").unwrap().clone();
        let orange = scope.get("orange").unwrap().clone();
        let yellow = scope.get("yellow").unwrap().clone();
        let green = scope.get("green").unwrap().clone();
        let blue = scope.get("blue").unwrap().clone();
        let purple = scope.get("purple").unwrap().clone();
        let base0 = scope.get("base0").unwrap().clone();
        let base1 = scope.get("base1").unwrap().clone();
        let base2 = scope.get("base2").unwrap().clone();
        let base3 = scope.get("base3").unwrap().clone();
        let base4 = scope.get("base4").unwrap().clone();
        let base5 = scope.get("base5").unwrap().clone();
        let base6 = scope.get("base6").unwrap().clone();
        let base7 = scope.get("base7").unwrap().clone();
        let base8 = scope.get("base8").unwrap().clone();

        commands.insert_resource(ColorTemplate {
            red: red.cast::<typst::visualize::Color>().unwrap(),
            orange: orange.cast::<typst::visualize::Color>().unwrap(),
            yellow: yellow.cast::<typst::visualize::Color>().unwrap(),
            green: green.cast::<typst::visualize::Color>().unwrap(),
            blue: blue.cast::<typst::visualize::Color>().unwrap(),
            purple: purple.cast::<typst::visualize::Color>().unwrap(),
            base0: base0.cast::<typst::visualize::Color>().unwrap(),
            base1: base1.cast::<typst::visualize::Color>().unwrap(),
            base2: base2.cast::<typst::visualize::Color>().unwrap(),
            base3: base3.cast::<typst::visualize::Color>().unwrap(),
            base4: base4.cast::<typst::visualize::Color>().unwrap(),
            base5: base5.cast::<typst::visualize::Color>().unwrap(),
            base6: base6.cast::<typst::visualize::Color>().unwrap(),
            base7: base7.cast::<typst::visualize::Color>().unwrap(),
            base8: base8.cast::<typst::visualize::Color>().unwrap(),
        });

        let frame = scope.get("frame").unwrap().clone();
        let button = scope.get("button").unwrap().clone();
        let icon = scope.get("icon").unwrap().clone();

        commands.insert_resource(Template {
            frame: frame.cast::<typst::foundations::Func>().unwrap(),
            button: button.cast::<typst::foundations::Func>().unwrap(),
            icon: icon.cast::<typst::foundations::Func>().unwrap(),
        });
    }
}

fn ui_update(
    mut q_scene: Query<&mut VelloScene>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    world: Res<TypstCompiler>,
    time: Res<Time>,
    template: Res<Template>,
    color_template: Res<ColorTemplate>,
) {
    let world = world.world_meta();
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

    let mut writer = SimpleWriter::new();

    let frame = world.eval_func::<Content>(&template.frame, [text("Frame example")]);
    let button = world.eval_func::<Content>(&template.button, [text("Button example")]);

    writer.blank_page(|writer| {
        writer.add_content(
            sequence!(
                heading(text(time.elapsed_seconds().to_string()))
                    .pack()
                    .styled(text::TextElem::set_fill(solid(color_template.blue))),
                text((1.0 / time.delta_seconds()).to_string()),
                linebreak(),
                frame.clone(),
                linebreak(),
                frame.clone(),
                linebreak(),
                frame.clone(),
                linebreak(),
                frame.clone(),
                linebreak(),
                button
            )
            .pack()
            .aligned(layout::Alignment::Both(
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

    // let context_elem = typst::foundations::ContextElem::new(template.frame.clone()).pack();

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
