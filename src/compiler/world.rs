use std::{
    fs, mem,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use bevy::{prelude::*, utils::HashMap};
use chrono::{DateTime, Datelike, Local};
use comemo::{Prehashed, Track, Validate};
use typst::{
    diag::{warning, FileError, FileResult, SourceResult},
    engine::{Engine, Route},
    eval::Tracer,
    foundations::{
        Bytes, Content, Context, Datetime, FromValue, Func, IntoArgs, Module, StyleChain,
    },
    introspection::{Introspector, Locator},
    layout::LayoutRoot,
    model::Document,
    syntax::{FileId, Source, Span},
    text::{Font, FontBook},
    Library, World,
};

use super::fonts::{FontSearcher, FontSlot};
use super::package;

/// Metadata for [`TypstWorldRef`].
pub struct TypstWorldMeta {
    /// The root relative to which absolute paths are resolved.
    root: PathBuf,
    /// Typst's standard library.
    library: Prehashed<Library>,
    /// Metadata about discovered fonts.
    book: Prehashed<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Vec<FontSlot>,
    /// Maps file ids to source files and buffers.
    slots: Mutex<HashMap<FileId, FileSlot>>,
    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceLock<DateTime<Local>>,
}

impl TypstWorldMeta {
    pub fn new(root: PathBuf, font_paths: &[PathBuf]) -> Self {
        let mut searcher = FontSearcher::default();
        searcher.search(font_paths);

        Self {
            root,
            library: Prehashed::new(Library::default()),
            book: Prehashed::new(searcher.book),
            fonts: searcher.fonts,
            slots: Mutex::new(HashMap::new()),
            now: OnceLock::new(),
        }
    }

    pub fn empty_world(&self) -> TypstWorldRef {
        TypstWorldRef {
            main: Source::detached(""),
            meta: self,
        }
    }

    pub fn world(&self, main: Source) -> TypstWorldRef<'_> {
        TypstWorldRef { main, meta: self }
    }

    pub fn world_from_str(&self, text: impl Into<String>) -> TypstWorldRef<'_> {
        self.world(Source::detached(text))
    }

    pub fn eval_str(&self, text: impl Into<String>) -> SourceResult<Module> {
        // Typst world
        let world: &dyn World = &self.world_from_str(text);
        let mut tracer = Tracer::new();

        // Try to evaluate the source file into a module.
        typst::eval::eval(
            world.track(),
            Route::default().track(),
            tracer.track_mut(),
            &world.main(),
        )
    }

    pub fn compile_str(&self, text: impl Into<String>) -> SourceResult<Document> {
        // Typst world
        let world = self.world_from_str(text);
        let mut tracer = Tracer::new();

        // Compile document
        let document = typst::compile(&world, &mut tracer)?;

        // Logs out typst warnings
        let warnings = tracer.warnings();
        if warnings.is_empty() == false {
            warn!("[Typst compilation warning]: {:#?}", warnings);
        }

        Ok(document)
    }

    /// Create a temporary [`Engine`] from the world for Typst evalulation.
    pub fn scoped_engine<T>(&self, scope: impl FnOnce(&mut Engine) -> T) -> T {
        let world: &dyn World = &self.empty_world();

        let document = Document::default();
        let constraint = <Introspector as Validate>::Constraint::new();
        let mut locator = Locator::new();
        let mut tracer = Tracer::new();

        let mut engine = Engine {
            world: world.track(),
            introspector: document.introspector.track_with(&constraint),
            route: Route::default(),
            locator: &mut locator,
            tracer: tracer.track_mut(),
        };

        scope(&mut engine)
    }

    // Referenced from: https://github.com/typst/typst/blob/v0.11.1/crates/typst/src/lib.rs#L107-L165
    // TODO: This should be implemented upstreamed (or at least exposed as pub fn)
    /// Compile [`Content`] into a [`Document`].
    pub fn compile_content(&self, content: Content) -> SourceResult<Document> {
        let world: &dyn World = &self.empty_world();
        let style_chain = StyleChain::new(&world.library().styles);

        let mut document = Document::default();
        let mut tracer = Tracer::new();
        let mut iter = 0;

        // Relayout until all introspections stabilize.
        // If that doesn't happen within five attempts, we give up.
        loop {
            // Clear delayed errors.
            tracer.delayed();

            let constraint = <Introspector as Validate>::Constraint::new();
            let mut locator = Locator::new();

            let mut engine = Engine {
                world: world.track(),
                introspector: document.introspector.track_with(&constraint),
                route: Route::default(),
                locator: &mut locator,
                tracer: tracer.track_mut(),
            };

            // Layout!
            document = content.layout_root(&mut engine, style_chain)?;
            document.introspector.rebuild(&document.pages);

            if document.introspector.validate(&constraint) {
                break;
            }

            iter += 1;
            if iter >= 5 {
                tracer.warn(warning!(
                    Span::detached(), "layout did not converge within 5 attempts";
                    hint: "check if any states or queries are updating themselves"
                ));
                break;
            }
        }

        // Promote delayed errors.
        let delayed = tracer.delayed();
        if !delayed.is_empty() {
            return Err(delayed);
        }

        Ok(document)
    }
}

pub struct TypstWorldRef<'a> {
    /// The input path.
    pub main: Source,
    meta: &'a TypstWorldMeta,
}

impl<'a> TypstWorldRef<'a> {
    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.meta.slots.lock().unwrap();
        f(map.entry(id).or_insert_with(|| FileSlot::new(id)))
    }
}

impl<'a> World for TypstWorldRef<'a> {
    fn library(&self) -> &Prehashed<Library> {
        &self.meta.library
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.meta.book
    }

    fn main(&self) -> Source {
        self.main.clone()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id, |slot| slot.source(&self.meta.root))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(&self.meta.root))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.meta.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = self.meta.now.get_or_init(chrono::Local::now);

        let naive = match offset {
            None => now.naive_local(),
            Some(o) => now.naive_utc() + chrono::Duration::hours(o),
        };

        Datetime::from_ymd(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
        )
    }
}

/// Holds the processed data for a file ID.
///
/// Both fields can be populated if the file is both imported and read().
struct FileSlot {
    /// The slot's file id.
    id: FileId,
    /// The lazily loaded and incrementally updated source file.
    source: SlotCell<Source>,
    /// The lazily loaded raw byte buffer.
    file: SlotCell<Bytes>,
}

impl FileSlot {
    /// Create a new path slot.
    fn new(id: FileId) -> Self {
        Self {
            id,
            file: SlotCell::new(),
            source: SlotCell::new(),
        }
    }

    /// Retrieve the source for this file.
    fn source(&mut self, project_root: &Path) -> FileResult<Source> {
        self.source.get_or_init(
            || system_path(project_root, self.id),
            |data, prev| {
                let text = decode_utf8(&data)?;
                if let Some(mut prev) = prev {
                    prev.replace(text);
                    Ok(prev)
                } else {
                    Ok(Source::new(self.id, text.into()))
                }
            },
        )
    }

    /// Retrieve the file's bytes.
    fn file(&mut self, project_root: &Path) -> FileResult<Bytes> {
        self.file.get_or_init(
            || system_path(project_root, self.id),
            |data, _| Ok(data.into()),
        )
    }
}

/// Lazily processes data for a file.
struct SlotCell<T> {
    /// The processed data.
    data: Option<FileResult<T>>,
    /// A hash of the raw file contents / access error.
    fingerprint: u128,
    /// Whether the slot has been accessed in the current compilation.
    accessed: bool,
}

impl<T: Clone> SlotCell<T> {
    /// Creates a new, empty cell.
    fn new() -> Self {
        Self {
            data: None,
            fingerprint: 0,
            accessed: false,
        }
    }

    /// Gets the contents of the cell or initialize them.
    fn get_or_init(
        &mut self,
        path: impl FnOnce() -> FileResult<PathBuf>,
        f: impl FnOnce(Vec<u8>, Option<T>) -> FileResult<T>,
    ) -> FileResult<T> {
        // If we accessed the file already in this compilation, retrieve it.
        if mem::replace(&mut self.accessed, true) {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        // Read and hash the file.
        let result = path().and_then(|p| read(&p));
        let fingerprint = typst::util::hash128(&result);

        // If the file contents didn't change, yield the old processed data.
        if mem::replace(&mut self.fingerprint, fingerprint) == fingerprint {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        let prev = self.data.take().and_then(Result::ok);
        let value = result.and_then(|data| f(data, prev));
        self.data = Some(value.clone());

        value
    }
}

/// Resolves the path of a file id on the system, downloading a package if
/// necessary.
fn system_path(project_root: &Path, id: FileId) -> FileResult<PathBuf> {
    // Determine the root path relative to which the file path
    // will be resolved.
    let buf;
    let mut root = project_root;
    if let Some(spec) = id.package() {
        buf = package::prepare_package(spec)?;
        root = &buf;
    }

    // Join the path to the root. If it tries to escape, deny
    // access. Note: It can still escape via symlinks.
    id.vpath().resolve(root).ok_or(FileError::AccessDenied)
}

/// Read a file.
fn read(path: &Path) -> FileResult<Vec<u8>> {
    let f = |e| FileError::from_io(e, path);
    if fs::metadata(path).map_err(f)?.is_dir() {
        Err(FileError::IsDirectory)
    } else {
        fs::read(path).map_err(f)
    }
}

/// Decode UTF-8 with an optional BOM.
fn decode_utf8(buf: &[u8]) -> FileResult<&str> {
    // Remove UTF-8 BOM.
    Ok(std::str::from_utf8(
        buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf),
    )?)
}
