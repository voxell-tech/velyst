use std::marker::PhantomData;

use bevy::{color::palettes::css, prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_typst::{prelude::*, typst_element::prelude::*, TypstPlugin};
use bevy_vello::{prelude::*, VelloPlugin};
use typst_vello::TypstScene;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(TypstPlugin::default())
        .register_typst_asset::<GameUi>()
        .register_typst_func::<GameUi, MainFunc>()
        .register_typst_func::<GameUi, PerfMetricsFunc>()
        .render_typst_func::<MainFunc>()
        .add_systems(Startup, setup)
        .add_systems(Startup, load_typst_asset::<GameUi>)
        .add_systems(
            Update,
            main_func.run_if(resource_exists::<TypstContent<PerfMetricsFunc>>),
        )
        .add_systems(Startup, perf_metrics)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

struct GameUi;

impl TypstPath for GameUi {
    fn path() -> &'static str {
        "game_ui.typ"
    }
}

fn main_func(
    mut commands: Commands,
    q_window: Query<Ref<Window>, With<PrimaryWindow>>,
    q_interactions: Query<(&Interaction, &Name)>,
    perf_metrics: Res<TypstContent<PerfMetricsFunc>>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let mut btn_highlight = String::new();

    for (interaction, name) in q_interactions.iter() {
        match interaction {
            Interaction::Pressed => println!("pressed {name}"),
            Interaction::Hovered => println!("hovered {name}"),
            Interaction::None => {}
        }
        if *interaction == Interaction::Hovered {
            btn_highlight = name.to_string();

            break;
        }
    }

    if window.is_changed() || perf_metrics.is_changed() {
        commands.insert_resource(MainFunc {
            width: Abs::pt(window.width() as f64),
            height: Abs::pt(window.height() as f64),
            perf_metrics: perf_metrics.clone(),
            btn_highlight,
        });
    }
}

fn perf_metrics(mut commands: Commands, time: Res<Time>) {
    let fps = (1.0 / time.delta_seconds_f64() * 100.0).round() / 100.0;
    let elapsed_time = (time.elapsed_seconds_f64() * 100.0).round() / 100.0;

    commands.insert_resource(PerfMetricsFunc { fps, elapsed_time });
}

#[derive(Resource)]
pub struct MainFunc {
    width: Abs,
    height: Abs,
    perf_metrics: Content,
    btn_highlight: String,
}

impl TypstFunc for MainFunc {
    fn func_name(&self) -> &str {
        "main"
    }

    fn content(&self, func: foundations::Func) -> Content {
        context(func, |args| {
            args.push(self.width);
            args.push(self.height);
            args.push(self.perf_metrics.clone());
            args.push_named("btn_highlight", self.btn_highlight.clone());
        })
        .pack()
    }
}

#[derive(Resource)]
pub struct PerfMetricsFunc {
    fps: f64,
    elapsed_time: f64,
}

impl TypstFunc for PerfMetricsFunc {
    fn func_name(&self) -> &str {
        "perf_metrics"
    }

    fn content(&self, func: foundations::Func) -> Content {
        context(func, |args| {
            args.push(self.fps);
            args.push(self.elapsed_time);
        })
        .pack()
    }
}

pub trait TypstCommandExt {
    fn register_typst_asset<P: TypstPath>(&mut self) -> &mut Self;

    fn register_typst_func<P: TypstPath, F: TypstFunc>(&mut self) -> &mut Self;

    fn render_typst_func<F: TypstFunc>(&mut self) -> &mut Self;
}

impl TypstCommandExt for App {
    fn register_typst_asset<P: TypstPath>(&mut self) -> &mut Self {
        self.add_systems(PreStartup, load_typst_asset::<P>)
            .add_systems(Update, asset_change_detection::<P>)
    }

    fn register_typst_func<P: TypstPath, F: TypstFunc>(&mut self) -> &mut Self {
        self.add_systems(
            Update,
            update_typst_content::<P, F>.run_if(
                // Asset and function needs to exists first.
                resource_exists::<TypstAssetHandle<P>>
                    .and_then(resource_exists::<F>)
                    .and_then(
                        // Any changes to the asset or the function will cause a content update.
                        resource_changed::<TypstAssetHandle<P>>.or_else(resource_changed::<F>),
                    ),
            ),
        )
    }

    fn render_typst_func<F: TypstFunc>(&mut self) -> &mut Self {
        self.init_resource::<TypstSceneRef<F>>().add_systems(
            Update,
            (
                layout_typst_content::<F>.run_if(resource_exists_and_changed::<TypstContent<F>>),
                render_typst_scene::<F>.run_if(resource_exists_and_changed::<TypstSceneRef<F>>),
                construct_interaction_tree::<F>
                    .run_if(resource_exists_and_changed::<TypstSceneRef<F>>),
            ),
        )
    }
}

pub fn update_typst_content<P: TypstPath, F: TypstFunc>(
    mut commands: Commands,
    context: TypstContext<P>,
    func: Res<F>,
) {
    if let Some(scope) = context.get_scope() {
        let content = func.call_func(scope);
        commands.insert_resource(TypstContent::<F>::new(content));
    } else if context.is_loaded() {
        error!("Unable to get scope for #{}().", func.func_name());
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct TypstContent<F: TypstFunc>(#[deref] Content, PhantomData<F>);

impl<F: TypstFunc> TypstContent<F> {
    pub fn new(content: Content) -> Self {
        Self(content, PhantomData)
    }
}

impl<F: TypstFunc> Default for TypstContent<F> {
    fn default() -> Self {
        Self(Content::default(), PhantomData)
    }
}

pub fn load_typst_asset<P: TypstPath>(mut commands: Commands, asset_server: Res<AssetServer>) {
    let typst_handle = TypstAssetHandle::<P>::new(asset_server.load(P::path()));
    commands.insert_resource(typst_handle);
}

fn asset_change_detection<P: TypstPath>(
    mut asset_evr: EventReader<AssetEvent<TypstAsset>>,
    mut typst_handle: ResMut<TypstAssetHandle<P>>,
) {
    for asset_evt in asset_evr.read() {
        match asset_evt {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let handle = typst_handle.clone_weak();
                if *id == handle.id() {
                    typst_handle.set_changed();
                }
            }
            _ => {}
        }
    }
}

/// System implementation for layouting [`TypstFunc`] into [`TypstSceneRef`].
fn layout_typst_content<F: TypstFunc>(
    content: Res<TypstContent<F>>,
    world: Res<TypstWorldRef>,
    mut scene: ResMut<TypstSceneRef<F>>,
) {
    match world.layout_frame(&content) {
        Ok(frame) => {
            let new_scene = TypstScene::from_frame(&frame);
            **scene = new_scene;
        }
        Err(err) => error!("{err:#?}"),
    }
}

/// System implementation for rendering [`TypstSceneRef`] into [`VelloScene`].
fn render_typst_scene<F: TypstFunc>(
    mut commands: Commands,
    mut q_scenes: Query<&mut VelloScene>,
    mut typst_scene: ResMut<TypstSceneRef<F>>,
) {
    let typst_scene = typst_scene.bypass_change_detection();

    if let Some(mut scene) = typst_scene.entity.and_then(|e| q_scenes.get_mut(e).ok()) {
        **scene = typst_scene.render();
    } else {
        typst_scene.entity = Some(
            commands
                .spawn(VelloSceneBundle {
                    scene: typst_scene.render().into(),
                    coordinate_space: CoordinateSpace::ScreenSpace,
                    ..default()
                })
                .insert(NodeBundle::default())
                .id(),
        );
    }
}

/// Construct the interaction tree using bevy ui nodes.
fn construct_interaction_tree<F: TypstFunc>(
    mut commands: Commands,
    typst_scene: Res<TypstSceneRef<F>>,
) {
    let Some(root_entity) = typst_scene.entity else {
        return;
    };

    commands.entity(root_entity).despawn_descendants();

    let mut entities = Vec::with_capacity(typst_scene.groups_len());

    for group in typst_scene.iter_groups() {
        let parent_entity = match group.parent {
            Some(index) => entities[index],
            None => root_entity,
        };

        let coeffs = group.transform.as_coeffs();
        let translation = Vec2::new(coeffs[4] as f32, coeffs[5] as f32);
        let scale = Vec3::new(coeffs[0] as f32, coeffs[3] as f32, 0.0);

        let entity = commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(group.size.x as f32),
                    height: Val::Px(group.size.y as f32),
                    left: Val::Px(translation.x),
                    top: Val::Px(translation.y),
                    ..default()
                },
                transform: Transform::from_scale(scale),
                // background_color: css::RED.with_alpha(0.2).into(),
                ..default()
            })
            .set_parent(parent_entity)
            .id();

        if let Some(label) = group.label() {
            commands
                .entity(entity)
                .insert(Interaction::None)
                .insert(Name::new(label.as_str()));
        }

        entities.push(entity);
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TypstContext<'w, P: TypstPath> {
    pub handle: Res<'w, TypstAssetHandle<P>>,
    pub assets: Res<'w, Assets<TypstAsset>>,
}

impl<P: TypstPath> TypstContext<'_, P> {
    pub fn is_loaded(&self) -> bool {
        self.assets.contains(&**self.handle)
    }

    pub fn get_scope(&self) -> Option<&foundations::Scope> {
        self.assets
            .get(&**self.handle)
            .map(|asset| asset.module().scope())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct TypstAssetHandle<P: TypstPath>(#[deref] Handle<TypstAsset>, PhantomData<P>);

impl<P: TypstPath> TypstAssetHandle<P> {
    pub fn new(handle: Handle<TypstAsset>) -> Self {
        Self(handle, PhantomData)
    }
}

/// Reference storage of a [`TypstScene`] in a resource.
#[derive(Resource, Deref, DerefMut)]
pub struct TypstSceneRef<F> {
    #[deref]
    /// Underlying [`TypstScene`] data.
    scene: TypstScene,
    /// Entity that contains [`VelloSceneBundle`] for rendering the typst scene.
    entity: Option<Entity>,
    // node_overrides: HashMap<TypLabel, >
    phantom: PhantomData<F>,
}

impl<F> Default for TypstSceneRef<F> {
    fn default() -> Self {
        Self {
            scene: TypstScene::default(),
            entity: None,
            phantom: PhantomData,
        }
    }
}

impl<F> TypstSceneRef<F> {
    pub fn new(typst_scene: TypstScene) -> Self {
        Self {
            scene: typst_scene,
            entity: None,
            phantom: PhantomData,
        }
    }
}

pub trait TypstFunc: Resource {
    fn func_name(&self) -> &str;

    fn content(&self, func: foundations::Func) -> Content;

    fn call_func(&self, scope: &foundations::Scope) -> Content {
        match scope.get_func(self.func_name()) {
            Ok(func) => self.content(func),
            Err(err) => {
                warn!("{err:#?}");
                Content::empty()
            }
        }
    }

    fn layout_frame(
        &self,
        world: &TypstWorld,
        scope: &foundations::Scope,
    ) -> SourceResult<layout::Frame> {
        world.layout_frame(&self.call_func(scope))
    }
}

pub trait TypstPath: Send + Sync + 'static {
    fn path() -> &'static str;
}
