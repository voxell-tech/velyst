use std::marker::PhantomData;

use crate::{prelude::*, typst_element::prelude::*};
use bevy::{prelude::*, utils::HashMap};
use bevy_vello::prelude::*;
use typst_vello::TypstScene;

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
    /// Layout [`Content`] into a [`TypstScene`] which gets stored inside [`TypstSceneRef`].
    Layout,
    /// Render [`TypstScene`] into a [`VelloScene`].
    Render,
}

pub trait TypstCommandExt {
    /// Load [`TypstAsset`] using [`TypstPath::path()`] and detect changes made towards the asset.
    fn register_typst_asset<P: TypstPath>(&mut self) -> &mut Self;

    /// Compile [`TypstFunc`] into [`TypstContent`].
    fn compile_typst_func<P: TypstPath, F: TypstFunc>(&mut self) -> &mut Self;

    /// Layout [`TypstContent`] into [`TypstSceneRef`] and render it into a [`VelloScene`].
    fn render_typst_func<F: TypstFunc>(&mut self) -> &mut Self;
}

impl TypstCommandExt for App {
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
        self.add_systems(
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
        self.init_resource::<TypstSceneRef<F>>().add_systems(
            Update,
            (
                // Layout
                layout_typst_content::<F>
                    .run_if(resource_exists_and_changed::<TypstContent<F>>)
                    .in_set(VelystSet::Layout),
                // Render
                (render_typst_scene::<F>, construct_interaction_tree::<F>)
                    .run_if(resource_exists_and_changed::<TypstSceneRef<F>>)
                    .in_set(VelystSet::Render),
            ),
        )
    }
}

fn load_typst_asset<P: TypstPath>(mut commands: Commands, asset_server: Res<AssetServer>) {
    let typst_handle = TypstAssetHandle::<P>::new(asset_server.load(P::path()));
    commands.insert_resource(typst_handle);
}

fn compile_typst_func<P: TypstPath, F: TypstFunc>(
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
    mut typst_scene: ResMut<TypstSceneRef<F>>,
) {
    match world.layout_frame(&content) {
        Ok(frame) => {
            let new_scene = TypstScene::from_frame(&frame);
            typst_scene.set_scene(new_scene);
        }
        Err(err) => error!("{err:#?}"),
    }
}

/// System implementation for rendering [`TypstSceneRef`] into [`VelloScene`].
fn render_typst_scene<F: TypstFunc>(
    mut commands: Commands,
    mut q_scenes: Query<(&mut VelloScene, &mut Style)>,
    mut typst_scene: ResMut<TypstSceneRef<F>>,
) {
    let typst_scene = typst_scene.bypass_change_detection();

    if let Some((mut scene, mut style)) = typst_scene.entity.and_then(|e| q_scenes.get_mut(e).ok())
    {
        **scene = typst_scene.render();
        let size = typst_scene.size();
        style.width = Val::Px(size.x as f32);
        style.height = Val::Px(size.y as f32);
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
    mut q_nodes: Query<(&mut Style, &mut Transform, &mut ZIndex)>,
    mut typst_scene: ResMut<TypstSceneRef<F>>,
) {
    let Some(root_entity) = typst_scene.entity else {
        return;
    };

    typst_scene.reset_cached_entities_to_unused();
    let mut computed_transforms = Vec::with_capacity(typst_scene.groups_len());

    for i in 0..typst_scene.groups_len() {
        let group = typst_scene.get_group(i);

        // Calculate accumulated transform from the group hierarchy.
        let transform = match group.parent {
            Some(parent_index) => {
                let transform = computed_transforms[parent_index] * group.transform;
                computed_transforms.push(transform);
                transform
            }
            None => {
                computed_transforms.push(group.transform);
                group.transform
            }
        };

        let Some(label) = group.label() else {
            continue;
        };

        let coeffs = transform.as_coeffs();
        let left = Val::Px(coeffs[4] as f32);
        let top = Val::Px(coeffs[5] as f32);
        let width = Val::Px(group.size.x as f32);
        let height = Val::Px(group.size.y as f32);
        let scale = Vec3::new(coeffs[0] as f32, coeffs[3] as f32, 0.0);

        if let Some((mut style, mut transform, mut z_index)) = typst_scene
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

            match typst_scene.cached_entities.get_mut(&label) {
                Some(entities) => {
                    entities.push((new_entity, true));
                }
                None => {
                    typst_scene
                        .cached_entities
                        .insert(label, vec![(new_entity, true)]);
                }
            }
        }
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

/// Reference storage of a [`TypstScene`] in a resource.
#[derive(Resource, Deref, DerefMut)]
pub struct TypstSceneRef<F> {
    #[deref]
    /// Underlying [`TypstScene`] data.
    scene: TypstScene,
    /// Entity that contains [`VelloSceneBundle`] for rendering the typst scene.
    entity: Option<Entity>,
    /// Cached entities mapped by [`TypLabel`].
    ///
    /// - First element stores the cached entity itself.
    /// - Second element denotes whether it's used up or not.
    cached_entities: HashMap<TypLabel, Vec<(Entity, bool)>>, // TODO: Use SmallVec
    phantom: PhantomData<F>,
}

#[derive(Component, Deref, DerefMut)]
pub struct TypstLabel(TypLabel);

impl<F> Default for TypstSceneRef<F> {
    fn default() -> Self {
        Self {
            scene: default(),
            entity: None,
            cached_entities: default(),
            phantom: PhantomData,
        }
    }
}

impl<F> TypstSceneRef<F> {
    pub fn new(scene: TypstScene) -> Self {
        Self { scene, ..default() }
    }

    pub fn set_scene(&mut self, scene: TypstScene) {
        self.scene = scene;
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
