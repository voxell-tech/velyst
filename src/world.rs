use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use std::{fs, mem};

use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use chrono::{DateTime, Datelike, Local, Timelike};
use fonts::TypstFonts;
use typst::comemo::{Constraint, Track};
use typst::diag::{
    FileError, FileResult, PackageError, Severity, SourceDiagnostic,
};
use typst::engine::{Engine, Route, Sink, Traced};
use typst::foundations::{
    Bytes, Content, Datetime, Module, StyleChain,
};
use typst::introspection::Introspector;
use typst::layout::{Frame, Region};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};

pub mod fonts;

pub struct VelystWorldPlugin;

impl Plugin for VelystWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TypstRoot>()
            .init_resource::<TypstLibrary>()
            .init_resource::<TypstFonts>()
            .init_resource::<TypstDateTime>()
            .init_resource::<TypstFileSlots>();

        app.add_systems(
            Update,
            // Date time only goes down to the second,
            // so we don't need to update every frame.
            update_date_time.run_if(on_timer(Duration::from_secs(1))),
        );
    }
}

/// Update [`TypstDateTime`].
fn update_date_time(mut date_time: ResMut<TypstDateTime>) {
    date_time.0 = chrono::Local::now();
}

/// The root folder of the Typst assets.
#[derive(Resource, Deref, DerefMut)]
pub struct TypstRoot(PathBuf);

impl Default for TypstRoot {
    fn default() -> Self {
        let default_path = AssetPlugin::default().file_path;
        let mut root_path = PathBuf::from(".");
        root_path.push(default_path);

        Self(root_path)
    }
}

/// Typst's standard library.
#[derive(Resource, Deref, DerefMut)]
pub struct TypstLibrary(LazyHash<Library>);

impl Default for TypstLibrary {
    fn default() -> Self {
        Self(LazyHash::new(Library::default()))
    }
}

/// The current datetime if requested. This is stored here to ensure it is
/// always the same within one frame. Reset between frames.
#[derive(Resource, Deref, DerefMut)]
pub struct TypstDateTime(DateTime<Local>);

impl Default for TypstDateTime {
    fn default() -> Self {
        Self(chrono::Local::now())
    }
}

/// Maps file ids to source files and buffers.
pub type FileSlots = HashMap<FileId, FileSlot>;

/// A [`Mutex`] holder of [`FileSlots`].
#[derive(Resource, Default, Deref, DerefMut)]
pub struct TypstFileSlots(Mutex<FileSlots>);

#[derive(SystemParam)]
pub struct VelystWorld<'w> {
    pub root: Res<'w, TypstRoot>,
    pub library: Res<'w, TypstLibrary>,
    pub fonts: Res<'w, TypstFonts>,
    pub date_time: Res<'w, TypstDateTime>,
    pub file_slots: Res<'w, TypstFileSlots>,
}

impl VelystWorld<'_> {
    pub fn eval_source(&self, source: &Source) -> Option<Module> {
        // Typst world
        let world: &dyn typst::World = self;
        let mut sink = Sink::new();

        // Try to evaluate the source file into a module.
        let module = typst_eval::eval(
            &typst::ROUTINES,
            world.track(),
            Traced::default().track(),
            sink.track_mut(),
            Route::default().track(),
            source,
        );

        match module {
            Ok(module) => {
                for warning in sink.warnings() {
                    log_diagnostic(warning);
                }

                Some(module)
            }
            Err(errors) => {
                error!("Evaluation failed for {:?}!", source.id());
                for error in errors {
                    log_diagnostic(error);
                }

                None
            }
        }
    }

    pub fn layout_frame(
        &self,
        content: &Content,
        region: Region,
    ) -> Option<Frame> {
        let world: &dyn typst::World = self;
        let styles = StyleChain::new(&world.library().styles);

        let introspector = Introspector::default();
        let constraint = Constraint::default();

        let traced = Traced::default();
        let mut sink = Sink::new();

        // Relayout until all introspections stabilize.
        // If that doesn't happen within five attempts, we give up.
        // TODO: Implement the loop to support counter & states.
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
                region,
            )
        };

        // Log delayed errors.
        for delay in sink.delayed() {
            log_diagnostic(delay);
        }

        match frame {
            Ok(frame) => {
                for warning in sink.warnings() {
                    log_diagnostic(warning);
                }

                Some(frame)
            }
            Err(errors) => {
                error!("Layout failed!");
                for error in errors {
                    log_diagnostic(error);
                }

                None
            }
        }
    }

    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut file_slots = self.file_slots.lock().unwrap();
        f(file_slots.entry(id).or_insert_with(|| FileSlot::new(id)))
    }
}

impl typst::World for VelystWorld<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.fonts.book
    }

    fn main(&self) -> FileId {
        unreachable!()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id, |slot| slot.source(&self.root))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(&self.root))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let naive = match offset {
            None => self.date_time.naive_local(),
            Some(o) => {
                self.date_time.naive_utc()
                    + chrono::Duration::hours(o)
            }
        };

        Datetime::from_ymd_hms(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
            naive.hour().try_into().ok()?,
            naive.minute().try_into().ok()?,
            naive.second().try_into().ok()?,
        )
    }
}

/// Holds the processed data for a file ID.
///
/// Both fields can be populated if the file is both imported and read().
pub struct FileSlot {
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

    /// Reset the accessed state of the file.
    pub fn reset(&mut self) {
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

    /// Reset the accessed state of the file.
    fn reset(&mut self) {
        self.accessed = false;
    }

    /// Gets the contents of the cell or initialize them.
    fn get_or_init(
        &mut self,
        path: impl FnOnce() -> FileResult<PathBuf>,
        f: impl FnOnce(Vec<u8>, Option<T>) -> FileResult<T>,
    ) -> FileResult<T> {
        // If we accessed the file already in this compilation, retrieve it.
        if mem::replace(&mut self.accessed, true)
            && let Some(data) = &self.data
        {
            return data.clone();
        }

        // Read and hash the file.
        let result = path().and_then(|p| read(&p));
        let fingerprint = typst::utils::hash128(&result);

        // If the file contents didn't change, yield the old processed data.
        if mem::replace(&mut self.fingerprint, fingerprint)
            == fingerprint
            && let Some(data) = &self.data
        {
            return data.clone();
        }

        let prev = self.data.take().and_then(Result::ok);
        let value = result.and_then(|data| f(data, prev));
        self.data = Some(value.clone());

        value
    }
}

/// Resolves the path of a file id on the system, downloading a package if
/// necessary.
fn system_path(
    project_root: &Path,
    id: FileId,
) -> FileResult<PathBuf> {
    // Determine the root path relative to which the file path
    // will be resolved.
    if id.package().is_some() {
        const PACKAGE_ERROR: &str = "Package (online) imports is not supported, please download manually and reference them via file paths.";

        return Err(FileError::Package(PackageError::Other(Some(
            PACKAGE_ERROR.into(),
        ))));
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

fn log_diagnostic(diagnostic: SourceDiagnostic) {
    let mut log_msg = String::new();
    log_msg.push('\n');
    log_msg.push_str(&diagnostic.message);
    log_msg.push('\n');
    log_msg.push_str(&format!("In file: {:?}", diagnostic.span.id()));
    log_msg.push('\n');
    log_msg.push_str(&format!("Trace: {:?}", diagnostic.trace));
    log_msg.push('\n');
    log_msg
        .push_str(&format!("Hints: {}", diagnostic.hints.join("\n")));

    match diagnostic.severity {
        Severity::Error => error!("{log_msg}"),
        Severity::Warning => warn!("{log_msg}"),
    }
}
