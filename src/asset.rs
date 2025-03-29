use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::utils::HashMap;
use typst::foundations::{Bytes, Module};
use typst::syntax::{FileId, Source, VirtualPath};

use crate::world::VelystWorld;

pub struct TypstAssetPlugin;

impl Plugin for TypstAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TypstSource>()
            .init_asset_loader::<TypstSourceLoader>()
            .init_asset::<TypstFile>()
            .init_asset_loader::<TypstFileLoader>()
            .init_resource::<SourceModules>()
            .add_systems(PreUpdate, eval_source);
    }
}

fn eval_source(
    world: VelystWorld,
    mut evr_asset_event: EventReader<AssetEvent<TypstSource>>,
    mut modules: ResMut<SourceModules>,
    sources: Res<Assets<TypstSource>>,
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

                match world.eval_source(&source.0) {
                    Ok(module) => {
                        modules.insert(*id, module);
                    }
                    Err(diagnostics) => {
                        for diag in diagnostics {
                            error!(
                                "Typst compilation error:\nMessage: {}\nFile: {:?}\nTrace: {:?}\nHints: {}",
                                diag.message,
                                diag.span.id(),
                                diag.trace,
                                diag.hints.join("\n")
                            );
                        }
                    }
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
pub struct SourceModules(HashMap<AssetId<TypstSource>, Module>);

/// A Typst [`Source`] file loaded from disk.
#[derive(Asset, TypePath, Deref)]
pub struct TypstSource(pub(super) Source);

#[derive(Default)]
pub struct TypstSourceLoader;

impl AssetLoader for TypstSourceLoader {
    type Asset = TypstSource;

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

        Ok(TypstSource(source))
    }

    fn extensions(&self) -> &[&str] {
        &["typ"]
    }
}

/// An arbitrary file required by Typst compiler,
/// stored in [`Bytes`] format.
#[derive(Asset, TypePath, Deref)]
pub struct TypstFile(Bytes);

#[derive(Default)]
pub struct TypstFileLoader;

impl AssetLoader for TypstFileLoader {
    type Asset = TypstFile;

    type Settings = ();

    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let source = Bytes::new(bytes);

        Ok(TypstFile(source))
    }
}
