use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use bevy::prelude::*;
use bevy_vello::vello_svg::usvg::fontdb::{Database, Source};
use typst::foundations::Bytes;
use typst::text::{Font, FontBook, FontInfo};
use typst::utils::LazyHash;

/// Searches for fonts.
#[derive(Resource, Debug, Clone)]
pub struct TypstFonts {
    /// Metadata about all discovered fonts.
    pub book: LazyHash<FontBook>,
    /// Slots that the fonts are loaded into.
    pub fonts: Vec<FontSlot>,
}

impl Default for TypstFonts {
    fn default() -> Self {
        let mut fonts = Self {
            book: LazyHash::new(FontBook::new()),
            fonts: Vec::new(),
        };

        fonts.search(&[]);
        fonts
    }
}

/// Holds details about the location of a font and lazily load the font itself.
#[derive(Debug, Clone)]
pub struct FontSlot {
    /// The path at which the font can be found on the system.
    path: PathBuf,
    /// The index of the font in its collection. Zero if the path does not point
    /// to a collection.
    index: u32,
    /// The lazily loaded font.
    font: OnceLock<Option<Font>>,
}

impl FontSlot {
    /// Get the font for this slot.
    pub fn get(&self) -> Option<Font> {
        self.font
            .get_or_init(|| {
                let data = Bytes::new(fs::read(&self.path).ok()?);
                Font::new(data, self.index)
            })
            .clone()
    }
}

impl TypstFonts {
    /// Search everything that is available.
    pub fn search(&mut self, font_paths: &[PathBuf]) {
        let mut db = Database::new();

        // Font paths have highest priority.
        for path in font_paths {
            db.load_fonts_dir(path);
        }

        // System fonts have second priority.
        db.load_system_fonts();

        for face in db.faces() {
            let path = match &face.source {
                Source::File(path) | Source::SharedFile(path, _) => {
                    path
                }
                // We never add binary sources to the database, so there
                // shouln't be any.
                Source::Binary(_) => continue,
            };

            let info = db
                .with_face_data(face.id, FontInfo::new)
                .expect("database must contain this font");

            if let Some(info) = info {
                self.book.push(info);
                self.fonts.push(FontSlot {
                    path: path.clone(),
                    index: face.index,
                    font: OnceLock::new(),
                });
            }
        }

        // Embedded fonts have lowest priority.
        #[cfg(feature = "embed-fonts")]
        self.add_embedded();
    }

    /// Add fonts that are embedded in the binary.
    #[cfg(feature = "embed-fonts")]
    fn add_embedded(&mut self) {
        for data in typst_assets::fonts() {
            let buffer = Bytes::new(data);
            for (i, font) in Font::iter(buffer).enumerate() {
                self.book.push(font.info().clone());
                self.fonts.push(FontSlot {
                    path: PathBuf::new(),
                    index: i as u32,
                    font: OnceLock::from(Some(font)),
                });
            }
        }
    }
}
