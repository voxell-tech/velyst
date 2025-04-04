use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, OnceLock, RwLock};
use std::{fs, mem};

use bevy::prelude::*;
use bevy::utils::HashMap;
use chrono::{DateTime, Datelike, Local};
use typst::comemo::{Track, Validate};
use typst::diag::{FileError, FileResult, SourceResult};
use typst::engine::{Engine, Route, Sink, Traced};
use typst::foundations::{func, Bytes, Content, Datetime, Module, StyleChain};
use typst::introspection::Introspector;
use typst::layout::{Abs, Axes, Frame, Region};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

use fonts::{FontSearcher, FontSlot};

pub mod fonts;

pub struct TypstWorldPlugin(pub TypstWorldRef);

impl Plugin for TypstWorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .add_systems(Update, update_global_time);
    }
}

static ELAPSED_SECS: LazyLock<RwLock<f64>> = LazyLock::new(|| RwLock::new(0.0));
static DELTA_SECS: LazyLock<RwLock<f64>> = LazyLock::new(|| RwLock::new(0.0));

#[func]
fn elapsed_secs() -> f64 {
    *ELAPSED_SECS.read().unwrap()
}

#[func]
fn delta_secs() -> f64 {
    *DELTA_SECS.read().unwrap()
}

fn update_global_time(time: Res<Time>) {
    *ELAPSED_SECS.write().unwrap() = time.elapsed_seconds_f64();
    *DELTA_SECS.write().unwrap() = time.delta_seconds_f64();
}

/// Resource reference to the underlying [`TypstWorld`].
#[derive(Resource, Deref, DerefMut, Clone)]
pub struct TypstWorldRef(Arc<RwLock<TypstWorld>>);

impl TypstWorldRef {
    pub fn new(world: TypstWorld) -> Self {
        Self(Arc::new(RwLock::new(world)))
    }
}

/// World for compiling Typst's [`Content`].
pub struct TypstWorld {
    /// The root relative to which absolute paths are resolved.
    root: PathBuf,
    /// Typst's standard library.
    pub library: LazyHash<Library>,
    /// Metadata about discovered fonts.
    book: LazyHash<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Vec<FontSlot>,
    /// Maps file ids to source files and buffers.
    slots: Mutex<HashMap<FileId, FileSlot>>,
    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceLock<DateTime<Local>>,
}

impl TypstWorld {
    pub fn new(root: PathBuf, font_paths: &[PathBuf]) -> Self {
        let mut searcher = FontSearcher::default();
        searcher.search(font_paths);

        let mut library = Library::default();
        let scope = library.global.scope_mut();
        scope.define_func::<elapsed_secs>();
        scope.define_func::<delta_secs>();

        Self {
            root,
            library: LazyHash::new(library),
            book: LazyHash::new(searcher.book),
            fonts: searcher.fonts,
            slots: Mutex::new(HashMap::new()),
            now: OnceLock::new(),
        }
    }

    pub fn eval_file(&self, path: &str, text: impl Into<String>) -> SourceResult<Module> {
        let source = Source::new(FileId::new(None, VirtualPath::new(path)), text.into());
        // Typst world
        let world: &dyn World = self;

        // Try to evaluate the source file into a module.
        let module = typst_eval::eval(
            &typst::ROUTINES,
            world.track(),
            Traced::default().track(),
            Sink::new().track_mut(),
            Route::default().track(),
            &source,
        );
        self.reset();

        module
    }

    pub fn layout_frame(&self, content: &Content) -> SourceResult<Frame> {
        let world: &dyn World = self;
        let styles = StyleChain::new(&world.library().styles);

        let introspector = Introspector::default();
        let constraint = <Introspector as Validate>::Constraint::new();
        let traced = Traced::default();
        let mut sink = Sink::new();

        // Relayout until all introspections stabilize.
        // If that doesn't happen within five attempts, we give up.
        let frame = {
            // Clear delayed errors.
            sink.delayed();

            let mut engine = Engine {
                routines: &typst::ROUTINES,
                world: world.track(),
                introspector: introspector.track_with(&constraint),
                traced: traced.track(),
                sink: sink.track_mut(),
                route: Route::default(),
            };

            let locator = typst::introspection::Locator::root();

            // Layout!
            (typst::ROUTINES.layout_frame)(
                &mut engine,
                content,
                locator,
                styles,
                Region::new(Axes::new(Abs::inf(), Abs::inf()), Axes::new(false, false)),
            )?
        };

        // Promote delayed errors.
        let delayed = sink.delayed();
        if !delayed.is_empty() {
            return Err(delayed);
        }

        Ok(frame)
    }

    fn reset(&self) {
        let mut slots = self.slots.lock().unwrap();

        for slot in slots.values_mut() {
            slot.reset();
        }
    }
}

impl TypstWorld {
    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.slots.lock().unwrap();
        f(map.entry(id).or_insert_with(|| FileSlot::new(id)))
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        unreachable!("There shouldn't be a main file.")
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id, |slot| slot.source(&self.root))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(&self.root))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = self.now.get_or_init(chrono::Local::now);

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
            source: SlotCell::new(),
            file: SlotCell::new(),
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
            |data, _| Ok(Bytes::new(data)),
        )
    }

    fn reset(&mut self) {
        self.source.reset();
        self.file.reset();
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
        let fingerprint = typst::utils::hash128(&result);

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

    fn reset(&mut self) {
        self.accessed = false;
    }
}

/// Resolves the path of a file id on the system, downloading a package if
/// necessary.
fn system_path(project_root: &Path, id: FileId) -> FileResult<PathBuf> {
    // Determine the root path relative to which the file path
    // will be resolved.
    if id.package().is_some() {
        return Err(FileError::Package(typst::diag::PackageError::Other(Some("Package (online) imports is not supported, please download manually and reference them via file paths.".into()))));
    }

    // Join the path to the root. If it tries to escape, deny
    // access. Note: It can still escape via symlinks.
    id.vpath()
        .resolve(project_root)
        .ok_or(FileError::AccessDenied)
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
