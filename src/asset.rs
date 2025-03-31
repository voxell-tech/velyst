use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::utils::HashMap;
use typst::foundations::Module;
use typst::syntax::{FileId, Source, VirtualPath};

use crate::world::VelystWorld;

pub struct TypstAssetPlugin;

impl Plugin for TypstAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VelystSource>()
            .init_asset_loader::<VelystSourceLoader>()
            .init_resource::<VelystModules>()
            .add_systems(PreUpdate, eval_source);
    }
}

fn eval_source(
    world: VelystWorld,
    mut evr_asset_event: EventReader<AssetEvent<VelystSource>>,
    mut modules: ResMut<VelystModules>,
    sources: Res<Assets<VelystSource>>,
) {
    let mut reset = false;

    for asset_event in evr_asset_event.read() {
        match asset_event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let Some(source) = sources.get(*id) else {
                    continue;
                };

                // Reset the file slots if this is the first compilation in this frame.
                if reset == false {
                    let mut file_slots = world.file_slots.lock().unwrap();
                    for slot in file_slots.values_mut() {
                        slot.reset()
                    }
                    reset = true;
                }

                if let Some(module) = world.eval_source(&source.0) {
                    modules.insert(*id, module);
                }
            }
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
                modules.remove(id);
            }
            AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct VelystModules(HashMap<AssetId<VelystSource>, Module>);

/// A Typst [`Source`] file loaded from disk.
#[derive(Asset, TypePath, Deref)]
pub struct VelystSource(pub(super) Source);

#[derive(Component, Deref, DerefMut)]
pub struct VelystSourceHandle(pub Handle<VelystSource>);

#[derive(Default)]
pub struct VelystSourceLoader;

impl AssetLoader for VelystSourceLoader {
    type Asset = VelystSource;

    type Settings = ();

    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut text = String::new();
        reader.read_to_string(&mut text).await?;

        let path = load_context.asset_path().to_string();
        let source = Source::new(FileId::new(None, VirtualPath::new(&path)), text);

        Ok(VelystSource(source))
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

// /// An arbitrary file required by Typst compiler,
// /// stored in [`Bytes`] format.
// #[derive(Asset, TypePath, Deref)]
// pub struct TypstFile(Bytes);

// #[derive(Default)]
// pub struct TypstFileLoader;

// impl AssetLoader for TypstFileLoader {
//     type Asset = TypstFile;

//     type Settings = ();

//     type Error = std::io::Error;

//     async fn load(
//         &self,
//         reader: &mut dyn Reader,
//         _settings: &Self::Settings,
//         _load_context: &mut LoadContext<'_>,
//     ) -> Result<Self::Asset, Self::Error> {
//         let mut bytes = Vec::new();
//         reader.read_to_end(&mut bytes).await?;

//         let source = Bytes::new(bytes);

//         Ok(TypstFile(source))
//     }
// }
