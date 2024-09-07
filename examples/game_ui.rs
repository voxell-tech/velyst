use std::marker::PhantomData;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_typst::{prelude::*, typst_element::prelude::*, TypstPlugin};
use bevy_vello::{prelude::*, VelloPlugin};
use typst_vello::TypstScene;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VelloPlugin::default()))
        .add_plugins(TypstPlugin::default())
        .add_plugins(TypstRenderPlugin::<GameUi, MainFunc>::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, load_typst_asset::<GameUi>)
        .add_systems(Update, main_func.run_if(resource_exists::<PerfMetricsFunc>))
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
    context: TypstContext<GameUi>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    perf_metrics: Res<PerfMetricsFunc>,
) {
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let Some(scope) = context.get_scope() else {
        return;
    };

    commands.insert_resource(MainFunc {
        width: Abs::pt(window.width() as f64),
        height: Abs::pt(window.height() as f64),
        perf_metrics: perf_metrics.call_func(scope),
    });
}

fn perf_metrics(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(PerfMetricsFunc {
        fps: 1.0 / time.delta_seconds_f64(),
        elapsed_time: time.elapsed_seconds_f64(),
    });
}

#[derive(Resource)]
pub struct MainFunc {
    width: Abs,
    height: Abs,
    perf_metrics: Content,
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

pub fn load_typst_asset<P: TypstPath>(mut commands: Commands, asset_server: Res<AssetServer>) {
    let typst_handle = TypstAssetHandle::<P>::new(asset_server.load(P::path()));
    commands.insert_resource(typst_handle);
}

pub struct TypstRenderPlugin<P: TypstPath, F: TypstFunc>(PhantomData<P>, PhantomData<F>);

impl<P: TypstPath, F: TypstFunc> Default for TypstRenderPlugin<P, F> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<P, F> Plugin for TypstRenderPlugin<P, F>
where
    P: TypstPath,
    F: TypstFunc,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<TypstSceneRef<F>>().add_systems(
            Update,
            (
                asset_change_detection::<P>.run_if(resource_exists::<TypstAssetHandle<P>>),
                layout_typst_func::<P, F>.run_if(
                    resource_exists::<F>
                        .and_then(resource_exists::<TypstAssetHandle<P>>)
                        .and_then(
                            // Any changes to the asset or the function  will cause a relayout.
                            resource_changed::<TypstAssetHandle<P>>.or_else(resource_changed::<F>),
                        ),
                ),
                render_typst_scene::<F>.run_if(resource_exists_and_changed::<TypstSceneRef<F>>),
            )
                .chain(),
        );
    }
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
fn layout_typst_func<P: TypstPath, F: TypstFunc>(
    context: TypstContext<P>,
    content: Res<F>,
    world: Res<TypstWorldRef>,
    mut scene: ResMut<TypstSceneRef<F>>,
) {
    if let Some(scope) = context.get_scope() {
        match content.layout_frame(&world, scope) {
            Ok(frame) => {
                let new_scene = TypstScene::from_frame(&frame);
                **scene = new_scene;
            }
            Err(err) => error!("{err:#?}"),
        }
    } else {
        error!("Unable to get scope for #{}().", content.func_name());
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
                .id(),
        );
    }

    // SAFETY: Typst scene should be created by the code above.
    let builder = commands.entity(
        typst_scene
            .entity
            .expect("Typst scene should be created above."),
    );

    // Construct the interaction tree
    for group in typst_scene.iter_groups() {
        // builder.push_children(|b|)
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct TypstContext<'w, P: TypstPath> {
    pub handle: Res<'w, TypstAssetHandle<P>>,
    pub assets: Res<'w, Assets<TypstAsset>>,
}

impl<P: TypstPath> TypstContext<'_, P> {
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
