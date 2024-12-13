use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use bevy_vello::prelude::*;
use smallvec::SmallVec;
use typst_vello::TypstScene;

use crate::prelude::*;
use crate::typst_element::prelude::*;

pub struct VelystRendererPlugin;

impl Plugin for VelystRendererPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                VelystSet::AssetLoading,
                VelystSet::Compile,
                VelystSet::Layout,
                VelystSet::Render,
            )
                .chain(),
        );
    }
}

/// Velyst rendering pipeline.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum VelystSet {
    /// Loading and reloading of [`TypstAsset`].
    AssetLoading,
    /// Compile [`TypstFunc`] into a [`TypstContent`].
    Compile,
    /// Layout [`Content`] into a [`TypstScene`] which gets stored inside [`VelystScene`].
    Layout,
    /// Render [`TypstScene`] into a [`VelloScene`].
    Render,
}

pub trait VelystAppExt {
    /// Load [`TypstAsset`] using [`TypstPath::path()`] and detect changes made towards the asset.
    fn register_typst_asset<P: TypstPath>(&mut self) -> &mut Self;

    /// Compile [`TypstFunc`] into [`TypstContent`].
    fn compile_typst_func<P: TypstPath, F: TypstFunc>(&mut self) -> &mut Self;

    /// Layout [`TypstContent`] into [`VelystScene`] and render it into a [`VelloScene`].
    fn render_typst_func<F: TypstFunc>(&mut self) -> &mut Self;
}

impl VelystAppExt for App {
    fn register_typst_asset<P: TypstPath>(&mut self) -> &mut Self {
        self.add_systems(
            PreStartup,
            load_typst_asset::<P>.in_set(VelystSet::AssetLoading),
        )
        .add_systems(
            Update,
            asset_change_detection::<P>.in_set(VelystSet::AssetLoading),
        )
    }

    fn compile_typst_func<P: TypstPath, F: TypstFunc>(&mut self) -> &mut Self {
        self.init_resource::<TypstContent<F>>().add_systems(
            Update,
            compile_typst_func::<P, F>
                .run_if(
                    // Asset and function needs to exists first.
                    resource_exists::<TypstAssetHandle<P>>
                        .and_then(resource_exists::<F>)
                        .and_then(
                            // Any changes to the asset or the function will cause a content recompilation.
                            resource_changed::<TypstAssetHandle<P>>.or_else(resource_changed::<F>),
                        ),
                )
                .in_set(VelystSet::Compile),
        )
    }

    fn render_typst_func<F: TypstFunc>(&mut self) -> &mut Self {
        self.init_resource::<VelystScene<F>>().add_systems(
            Update,
            (
                // Layout
                layout_typst_content::<F>
                    .run_if(resource_exists_and_changed::<TypstContent<F>>)
                    .in_set(VelystSet::Layout),
                // Render
                (render_velyst_scene::<F>, construct_interaction_tree::<F>)
                    .run_if(resource_exists_and_changed::<VelystScene<F>>)
                    .in_set(VelystSet::Render),
            ),
        )
    }
}

fn load_typst_asset<P: TypstPath>(mut commands: Commands, asset_server: Res<AssetServer>) {
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

/// System implementation for compiling [`TypstFunc`] into [`TypstContent`].
fn compile_typst_func<P: TypstPath, F: TypstFunc>(
    context: TypstContext<P>,
    mut content: ResMut<TypstContent<F>>,
    func: Res<F>,
) {
    if let Some(scope) = context.get_scope() {
        let new_content = func.compile(scope);
        **content = new_content;
    } else if context.is_loaded() {
        error!("Unable to get scope for #{}().", func.func_name());
    }
}

/// System implementation for layouting [`TypstContent`] into [`VelystScene`].
fn layout_typst_content<F: TypstFunc>(
    content: Res<TypstContent<F>>,
    world: Res<TypstWorldRef>,
    mut scene: ResMut<VelystScene<F>>,
) {
    // TODO: Optimize this system (currently the bottleneck).
    match world.layout_frame(&content) {
        Ok(frame) => {
            scene.update_frame(&frame);
        }
        Err(err) => error!("{err:#?}"),
    }
}

/// System implementation for rendering [`VelystScene`] into [`VelloScene`].
fn render_velyst_scene<F: TypstFunc>(
    mut commands: Commands,
    mut q_scenes: Query<(&mut VelloScene, &mut Style, &mut Visibility)>,
    mut scene: ResMut<VelystScene<F>>,
    func: Res<F>,
) {
    let scene = scene.bypass_change_detection();

    if let Some((mut vello_scene, mut style, mut viz)) =
        scene.entity.and_then(|e| q_scenes.get_mut(e).ok())
    {
        // Scene
        **vello_scene = scene.render();
        let size = scene.size();
        // Style
        style.width = Val::Px(size.x as f32);
        style.height = Val::Px(size.y as f32);
        // Visibility
        *viz = scene.visibility;
    } else {
        scene.entity = Some(
            commands
                .spawn(VelloSceneBundle {
                    scene: scene.render().into(),
                    coordinate_space: CoordinateSpace::ScreenSpace,
                    ..default()
                })
                .insert((
                    NodeBundle {
                        visibility: scene.visibility,
                        ..default()
                    },
                    func.render_layers(),
                    VelystSceneTag::<F>::default(),
                ))
                .id(),
        );
    }
}

/// Construct the interaction tree using bevy ui nodes.
fn construct_interaction_tree<F: TypstFunc>(
    mut commands: Commands,
    mut q_nodes: Query<(&mut Style, &mut Transform, &mut ZIndex, &mut Visibility)>,
    mut scene: ResMut<VelystScene<F>>,
) {
    let scene = scene.bypass_change_detection();

    let Some(root_entity) = scene.entity else {
        return;
    };

    scene.reset_cached_entities_to_unused();
    let mut computed_transforms = Vec::with_capacity(scene.groups_len());

    for i in 0..scene.groups_len() {
        let group = scene.get_group(i);

        // Calculate accumulated transform from the group hierarchy.
        let transform = match group.parent() {
            Some(parent_index) => {
                let transform = computed_transforms[parent_index] * group.transform();
                computed_transforms.push(transform);
                transform
            }
            None => {
                let transform = group.transform();
                computed_transforms.push(transform);
                transform
            }
        };

        let Some(label) = group.label() else {
            continue;
        };

        let coeffs = transform.as_coeffs();
        let left = Val::Px(coeffs[4] as f32);
        let top = Val::Px(coeffs[5] as f32);
        let width = Val::Px(group.size().x as f32);
        let height = Val::Px(group.size().y as f32);
        let scale = Vec3::new(coeffs[0] as f32, coeffs[3] as f32, 0.0);

        // Reuse cached nodes when available, otherwise, spawn a new one.
        if let Some((mut style, mut transform, mut z_index, mut viz)) = scene
            .cached_entities
            .get_mut(&label)
            .and_then(|entities| entities.iter_mut().find(|(_, used)| *used == false))
            .and_then(|(entity, used)| {
                *used = true;
                q_nodes.get_mut(*entity).ok()
            })
        {
            // Style
            style.left = left;
            style.top = top;
            style.width = width;
            style.height = height;
            // Scale
            transform.scale = scale;
            // ZIndex
            *z_index = ZIndex::Local(i as i32);
            // Visibility
            *viz = Visibility::Inherited;
        } else {
            let new_entity = commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left,
                            top,
                            width,
                            height,
                            ..default()
                        },
                        transform: Transform::from_scale(scale),
                        z_index: ZIndex::Local(i as i32),
                        ..default()
                    },
                    Interaction::default(),
                    TypstLabel(label),
                ))
                .set_parent(root_entity)
                .id();

            match scene.cached_entities.get_mut(&label) {
                Some(entities) => {
                    entities.push((new_entity, true));
                }
                None => {
                    scene
                        .cached_entities
                        .insert(label, SmallVec::from_buf([(new_entity, true)]));
                }
            }
        }
    }

    // Hide unused cached nodes
    for entities in scene.cached_entities.values() {
        for (entity, used) in entities {
            match *used {
                true => continue,
                false => {
                    commands.entity(*entity).insert(Visibility::Hidden);
                }
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct TypstAssetHandle<P: TypstPath>(#[deref] Handle<TypstAsset>, PhantomData<P>);

impl<P: TypstPath> TypstAssetHandle<P> {
    pub fn new(handle: Handle<TypstAsset>) -> Self {
        Self(handle, PhantomData)
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

/// A tag component for the entity that holds the [`VelloScene`]
/// that is rendering the [`VelystScene<F>`].
#[derive(Component, Debug)]
pub struct VelystSceneTag<F: TypstFunc>(PhantomData<F>);

impl<F: TypstFunc> Default for VelystSceneTag<F> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Storage of a [`TypstScene`] in a resource as well as
/// caching the render and interaction entities.
#[derive(Resource, Deref, DerefMut)]
pub struct VelystScene<F: TypstFunc> {
    #[deref]
    /// Underlying [`TypstScene`] data.
    scene: TypstScene,
    /// Visibility of the scene.
    pub visibility: Visibility,
    /// Entity that contains [`VelloSceneBundle`] for rendering the typst scene.
    entity: Option<Entity>,
    /// Cached entities mapped by [`TypLabel`].
    ///
    /// - First element stores the cached entity itself.
    /// - Second element denotes whether it's used up or not.
    cached_entities: HashMap<TypLabel, SmallVec<[(Entity, bool); 1]>>, // TODO: Use SmallVec
    phantom: PhantomData<F>,
}

impl<F: TypstFunc> VelystScene<F> {
    pub fn new(scene: TypstScene) -> Self {
        Self { scene, ..default() }
    }

    /// Resets all cached entities to unused.
    fn reset_cached_entities_to_unused(&mut self) {
        for entities in self.cached_entities.values_mut() {
            for (_, used) in entities.iter_mut() {
                *used = false;
            }
        }
    }
}

impl<F: TypstFunc> Default for VelystScene<F> {
    fn default() -> Self {
        Self {
            scene: TypstScene::default(),
            visibility: Visibility::Inherited,
            entity: None,
            cached_entities: default(),
            phantom: PhantomData,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct TypstLabel(TypLabel);

pub trait TypstFunc: Resource {
    fn func_name(&self) -> &str;

    fn render_layers(&self) -> RenderLayers {
        RenderLayers::layer(0)
    }

    fn content(&self, func: foundations::Func) -> Content;

    fn compile(&self, scope: &foundations::Scope) -> Content {
        match scope.get_func(self.func_name()) {
            Ok(func) => self.content(func),
            Err(err) => {
                error!("Compilation error on `{}`: {:#?}", self.func_name(), err);
                Content::empty()
            }
        }
    }
}

pub trait TypstPath: Send + Sync + 'static {
    fn path() -> &'static str;
}
