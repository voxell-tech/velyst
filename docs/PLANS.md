# Plan: kanva-native Typst→Kanva pipeline (`KanvaSink` + `kanva_typst` + `kanva_svg`)

## Context

Typst text/shapes styled with **both** fill and stroke currently produce **two `KanvaPath`s per element**. Root cause: the pipeline routes Typst through `imaging::PaintSink`, whose draw methods carry a peniko `Style` that is **fill XOR stroke** - so `render_text`/`render_shape` must emit two passes. This breaks per-path write-on animation (`TraceFadeKanva`): half the paths are fill-only (held invisible during the trace phase), so visible motion bunches at the end.

A merge-after-the-fact workaround is possible but is a band-aid. The real fix is to stop going through the fill-XOR-stroke abstraction: add a **kanva-native `KanvaSink` trait** that expresses **combined fill+stroke per element**, and a new **`kanva_typst`** crate that walks Typst frames straight into it. Then text/shapes emit a single path carrying both, and no doubling ever happens.

`imaging` is a read-only git dep, so its `PaintSink` can't change - but `kanva::KanvaPath` already has both `fill` and `stroke` fields (`kanva/src/node.rs:56-65`) and the renderer draws fill-then-stroke on one path (`kanva/src/lib.rs`, the `render` impl). So the data model already supports the goal; only the build path needs replacing.

### Type/import grounding (verify before coding - easy to get wrong)
`kanva` does **not** depend on `peniko`/`kurbo` directly; it re-exports them through the `imaging` dep. The kanva types are defined in `kanva/src/node.rs` as:
```rust
// imports at top of node.rs:
use imaging::kurbo::{Affine, BezPath, Stroke};
use imaging::peniko::{Brush, Fill, Style};
use imaging::{ClipRef, Composite};

pub struct KanvaPath  { pub path: BezPath, pub transform: Affine,
                        pub fill: Option<usize>, pub stroke: Option<usize> }
pub struct KanvaFill  { pub rule: Fill, pub brush: Brush,
                        pub brush_transform: Option<Affine>, pub composite: Composite }
pub struct KanvaStroke{ pub stroke: Stroke, pub brush: Brush,
                        pub brush_transform: Option<Affine>, pub composite: Composite }
pub struct KanvaClip  { pub path: BezPath, pub transform: Affine, pub style: Style }
// also: KanvaClip::from_ref(ClipRef<'_>) -> KanvaClip
```
So in the examples below: `Composite` is `imaging::Composite`, `Glyph` is `imaging::record::Glyph`, and `Fill`/`Brush`/`Style`/`Affine`/`BezPath`/`Stroke` come from `imaging::peniko`/`imaging::kurbo`. `typst_imaging` instead imports `peniko` and `imaging::record::Glyph` directly. These compile together **only because `kanva` and `typst_imaging` resolve to the same `peniko`/`imaging` versions** - `text_paint`'s returned `peniko::Brush` drops straight into `KanvaFill.brush` today via the existing `impl PaintSink for KanvaBuilder`, which proves the versions already match. `kanva_typst` must depend on those same versions (use `imaging`'s re-exports, or the same `peniko`/`imaging` git revs velyst already pins).

Decisions (from user): **reuse `typst_imaging`'s conversion helpers** (don't duplicate them); **native SVG/PDF → KanvaSink** (no `PaintSink` fallback) - scoped as a later phase since it's the heavy part.

## Architecture

### 1. `KanvaSink` trait - in the `kanva` crate (`kanva/src/sink.rs`, re-exported from `lib.rs`/prelude)
Uses kanva's own `KanvaFill`/`KanvaStroke`/`KanvaClip` types so the builder stores them with zero re-conversion.
```rust
pub trait KanvaSink {
    fn push_context(&mut self, label: &str);
    fn pop_context(&mut self);
    fn push_group(&mut self, clip: Option<KanvaClip>, composite: Composite);
    fn pop_group(&mut self);
    fn draw_path(&mut self, path: BezPath, transform: Affine,
                 fill: Option<KanvaFill>, stroke: Option<KanvaStroke>);   // combined
    fn glyph_run(&mut self, run: GlyphRun,
                 fill: Option<KanvaFill>, stroke: Option<KanvaStroke>,
                 glyphs: &mut dyn Iterator<Item = Glyph>);                 // combined
    // No `image` method: raster images are emitted via `draw_path` with a
    // `KanvaFill { brush: Brush::Image(..), .. }` over a rect path (see §2).
}
pub struct GlyphRun { pub font: FontData, pub transform: Affine,
                      pub glyph_transform: Option<Affine>, pub font_size: f32 }
```

### 2. `KanvaBuilder` implements `KanvaSink` (`kanva/src/builder.rs`)
`KanvaBuilder` already has the private helpers the new impl reuses: `push_path(KanvaPath) -> usize`, `push_group_entry(Group)`, `pop_group_entry()`, plus fields `kanva.{paths,fills,strokes,groups,commands,group_cmds,index}`, `group_stack`, `pending_label`. The new trait impl lives in the same crate so it calls these directly.
- `draw_path`: if `fill`/`stroke` present, push each into `kanva.fills`/`kanva.strokes` (recording the index), create **one** `KanvaPath { path, transform, fill: idx?, stroke: idx? }` via `push_path`, then `kanva.commands.push(Command::DrawPath(idx))`. (Mirror `fill()`/`stroke()` at `builder.rs:118-152` but combined.)
- `glyph_run`: outline each glyph **once** and attach both indices. The current `impl PaintSink`'s `glyph_run` (`builder.rs:154-228`) already does the ttf_parser outlining (`Face::parse`, `units_per_em`, `scale_tf`, the `GlyphPen: OutlineBuilder` at `builder.rs:235-269`) but emits fill **xor** stroke per glyph. **Factor that outlining into a shared helper**, then in the `KanvaSink` version: push one shared `KanvaFill` and/or one shared `KanvaStroke` for the whole run, and per glyph create **one** `KanvaPath` referencing both. Keep wrapping the run in a `push_group_entry`/`pop_group_entry` as today (composite now lives on `KanvaFill`/`KanvaStroke`, so the wrapping group can use `Group::default()`).
- `push_context`/`pop_context`: set/clear `pending_label` (identical to the `PaintSink` impl at `builder.rs:87-93`).
- `push_group`/`pop_group`: `push_group_entry(Group { clip, composite, ..default })` / `pop_group_entry()`.
- **Raster `image`:** a raster image is just a `Brush::Image` filling a rect - it can be expressed as `draw_path(rect_path, transform, Some(KanvaFill { brush: Brush::Image(..), .. }), None)` with **no dedicated method**. So the trait's `image` method is optional; prefer routing raster through `draw_path` and drop `image` from the trait unless a backend needs it.
- **Trait method-name collision:** `KanvaBuilder` will impl **both** `PaintSink` and `KanvaSink`, which share method names (`push_context`, `pop_context`, `push_group`, `pop_group`, `glyph_run`). Rust allows this; generic code bound by `impl KanvaSink` resolves unambiguously. **However** - after the velyst switch (§5), nothing feeds a `KanvaBuilder` through `PaintSink` anymore (see §5), so the cleanest option is to **delete `impl PaintSink for KanvaBuilder`** and keep only `KanvaSink`. That also removes the collision. (`typst_imaging`'s own `PaintSink`-based walker stays - it's used by a different sink; see §5.)

### 3. New crate `kanva_typst` (`velyst/crates/kanva_typst/`)
Mirrors `typst_imaging`'s structure but emits to `KanvaSink`. Depends on `kanva`, `typst-library`, `peniko`, and **`typst_imaging`** (for the pure conversion helpers). Add it to the workspace `members` in `velyst/Cargo.toml`.
- `lib.rs`: `pub fn render_frame(frame: &Frame, sink: &mut impl KanvaSink)` + the frame walker (`render_items`/`render_group`, Group/Text/Shape/Image dispatch, Link/Tag ignored - port `typst_imaging/src/lib.rs:14-102` swapping the sink calls). The group port maps `group.label` → `sink.push_context(&label.resolve())`, and a `group.clip` → a `KanvaClip { path: convert_curve(clip).to_path(tol), transform: state.transform, style: Style::Fill(NonZero) }` passed to `sink.push_group(Some(clip), Composite::default())`.
- **`RenderState`:** the walker needs `typst_imaging::RenderState`. It is currently `pub(crate)` and its hard-frame helper `pre_concat_container` (`lib.rs:153`) is **private**. Two options: (a) make `RenderState` + all its methods (incl. `pre_concat_container`) `pub` and reuse it; or (b) **copy** `RenderState` into `kanva_typst` (~55 lines, depends only on `convert::convert_transform`, which is already `pub`). Copying is self-contained and avoids widening `typst_imaging`'s API - recommended.
- `text.rs`, `shape.rs`: as shown below - reuse `text_paint`/`shape_paint`/`convert_*`, emit one combined call.
- `image.rs`: raster → `draw_path` with an image-brush `KanvaFill` (see §2). SVG/PDF → Phase 2 (skip + log in Phase 1).

### 4. Reuse `typst_imaging` helpers - required visibility changes (`typst_imaging`)
All return pure `peniko`/`kurbo` types, decoupled from `PaintSink`. Current state (verified):
- `mod convert`, `mod paint`, `mod image`, `mod shape`, `mod text` are **already `pub`** (`lib.rs:7-11`) - no change needed.
- `convert_transform`, `convert_fixed_stroke`, `convert_geometry`, `convert_curve` are **already `pub`**.
- `shape_paint` (`paint.rs:17`, returns `(Brush, Option<Affine>)`) and `text_paint` (`paint.rs:98`, returns `Brush`) are `pub(crate)` → **bump to `pub`**.
- `RenderState` is `pub(crate)` → bump to `pub` **only if** reusing it (Option (a) in §3); if copying it into `kanva_typst` (Option (b), recommended), no change to `typst_imaging` is needed here.
No logic changes - gradient/pattern/conic/text-gradient baking and geometry/stroke conversion are reused verbatim.

### 5. velyst integration - single switch (`velyst/crates/velyst/src/renderer.rs`)
`render_frame` (from `typst_imaging`) has **two** callers in `renderer.rs`:
1. `build_kanva_scene` (line ~237) - feeds a `KanvaBuilder`. **This is the only one we change**: replace `render_frame(frame, &mut builder)` with `kanva_typst::render_frame(frame, &mut builder)`. Add `kanva_typst` to velyst's `Cargo.toml` and update the `use` at `renderer.rs:10`.
2. `frame_to_scene` (line ~327, called at `renderer.rs:198` and `:220`) - renders entities **without** a `VelystKanva` straight to Vello via `VelloSceneSink` (a different `impl PaintSink`). **Leave this untouched.** It is why `typst_imaging::render_frame` and `typst_imaging`'s `PaintSink`-based walker stay alive - they are not dead after the switch.

Consequence for §2: after the switch, **no caller feeds a `KanvaBuilder` through `PaintSink`** (the kanva path now goes through `KanvaSink`; the direct-Vello path uses `VelloSceneSink`, not `KanvaBuilder`). So `impl PaintSink for KanvaBuilder` is dead and can be deleted (recommended) - `typst_imaging`'s own `PaintSink` impls for other sinks are unaffected.

> The two examples below are **illustrative** (eliding some `use`s/glyph-accumulation). Resolve `Composite`/`Glyph` from `imaging`/`imaging::record` and `Fill`/`Brush`/`Affine`/`BezPath`/`Stroke`/`FontData`/`Blob` from `peniko`/`peniko::kurbo` per the "Type/import grounding" note. `use typst_imaging::RenderState` applies to §3 Option (a); if you copied `RenderState` (Option (b), recommended), import the local one instead.

## Example: `kanva_typst/src/text.rs` (vs current two-pass `typst_imaging/text.rs:68-110`)
```rust
use typst_imaging::paint::text_paint;             // reused
use typst_imaging::convert::convert_fixed_stroke; // reused
use typst_imaging::RenderState;
use kanva::{KanvaSink, GlyphRun, KanvaFill, KanvaStroke};

pub(crate) fn render_text(text: &TextItem, sink: &mut impl KanvaSink, state: RenderState) {
    let font_data = FontData::new(Blob::new(Arc::new(text.font.data().clone())), text.font.index());
    let font_size = text.size.to_pt() as f32;
    let glyphs: Vec<Glyph> = /* same x-advance accumulation as typst_imaging text.rs:26-41 */;
    let Some(last) = glyphs.last() else { return };

    let fill = Some(KanvaFill { rule: Fill::NonZero,
        brush: text_paint(&text.fill, &state, last.x as f64),
        brush_transform: None, composite: Composite::default() });
    let stroke = text.stroke.as_ref().map(|s| KanvaStroke {
        stroke: convert_fixed_stroke(s),
        brush: text_paint(&s.paint, &state, last.x as f64),
        brush_transform: None, composite: Composite::default() });

    sink.glyph_run(GlyphRun { font: font_data, transform: state.transform,
                              glyph_transform: None, font_size },
                   fill, stroke, &mut glyphs.iter().copied());   // ONE call
}
```

## Example: `kanva_typst/src/shape.rs` (vs current `typst_imaging/shape.rs:19-48`)
```rust
use typst_imaging::paint::shape_paint;                               // reused
use typst_imaging::convert::{convert_fixed_stroke, convert_geometry}; // reused
use kanva::{KanvaSink, KanvaFill, KanvaStroke};

pub(crate) fn render_shape(shape: &viz::Shape, sink: &mut impl KanvaSink, state: RenderState) {
    let path = convert_geometry(&shape.geometry);
    let fill = shape.fill.as_ref().map(|paint| {
        let (brush, brush_transform) = shape_paint(paint, shape, &state);
        KanvaFill { rule: match shape.fill_rule { viz::FillRule::NonZero => Fill::NonZero,
                                                  viz::FillRule::EvenOdd => Fill::EvenOdd },
                    brush, brush_transform, composite: Composite::default() } });
    let stroke = shape.stroke.as_ref().map(|s| {
        let (brush, brush_transform) = shape_paint(&s.paint, shape, &state);
        KanvaStroke { stroke: convert_fixed_stroke(s), brush, brush_transform,
                      composite: Composite::default() } });

    sink.draw_path(path, state.transform, fill, stroke);   // ONE call
}
```

## `kanva_svg` crate (Phase 2 - native SVG/PDF → `KanvaSink`)

### Why a separate crate is needed
Typst embeds SVG and PDF as `Image` items. Today `typst_imaging/image.rs` handles them by:
- **SVG:** `SvgDocument::from_data(svg.data(), ..)` → `imaging::Painter::new(sink)` → `doc.render(&mut painter, RenderOptions { transform })`.
- **PDF:** `hayro_svg::convert(pdf.page(), ..)` → SVG string → `SvgDocument::from_str(..)` → same `Painter` render.

Both routes funnel through `svg_imaging`, which renders **into an `imaging::Painter` that drives `PaintSink`** - i.e. the same fill-XOR-stroke abstraction `kanva_typst` was built to escape. So an SVG `<path>` with both `fill` and `stroke` still emits **two** `KanvaPath`s, and `TraceFadeKanva` write-on over SVG content doubles exactly like text/shapes did before Phase 1. Phase 1's combined-path fix does **not** cover SVG/PDF because that content never touches `kanva_typst/text.rs`/`shape.rs` - it's lowered by `svg_imaging`.

`svg_imaging` lives **inside the read-only `imaging` git dep** (`imaging/svg_imaging/`), so it can't be changed and it can't be made to target `KanvaSink`. It parses with `usvg`, lowers to a crate-local render plan, and emits via `imaging::Painter`. Supported: path fills/strokes, gradients, paint order, isolated-group compositing, clip paths (incl. referenced chains), masks, `usvg`-flattened text, nested `<image>`, and raster PNG/JPEG/GIF/WebP. **Unsupported (reported, not drawn):** filters and pattern paints. It is `no_std + alloc` and text needs fonts loaded into `ParseOptions` (default db is empty).

### Design: native `usvg` walker → `KanvaSink`
`kanva_svg` is a new velyst crate that parses SVG with `usvg` directly (same parser `svg_imaging` uses) and walks the `usvg::Tree` straight into `KanvaSink`, emitting **one combined `draw_path`** per node (fill+stroke together). This mirrors what `kanva_typst` does for Typst frames, but for the usvg node tree.
```rust
pub fn render_svg(tree: &usvg::Tree, sink: &mut impl KanvaSink, transform: Affine);
// PDF reuses this: hayro_svg::convert(page) -> usvg::Tree::from_str -> render_svg(..)
```
- **Path node:** convert `usvg::Path` geometry → `BezPath`; build `Option<KanvaFill>` from `path.fill()` and `Option<KanvaStroke>` from `path.stroke()`; honor `paint_order` (kanva draws fill-then-stroke per `KanvaPath`, so paint-order-stroke-first SVGs need the two split into separate paths - the **only** case where we intentionally emit two paths).
- **Group node:** map to `push_group`/`pop_group` with the group's clip + opacity/composite; recurse. Emit `push_context`/`pop_context` with the node id so `KanvaGroup::inner(id)` can target SVG sub-trees for animation.
- **Image node:** raster → `draw_path` with an image-brush `KanvaFill` (as in §2, no `image()` method); nested SVG → recurse via `render_svg`.
- **Text:** `usvg` flattens text to paths when configured, so it arrives as path nodes (no separate text handling).
- **Gradients/clips/masks:** convert usvg paints/clip-paths to kanva equivalents. Filters & patterns: log unsupported (parity with `svg_imaging`).

### Integration
`kanva_typst/image.rs` dispatches `ImageKind::Svg`/`ImageKind::Pdf` to `kanva_svg::render_svg(..)` instead of the `svg_imaging`/`Painter` path. `kanva_svg` depends on `kanva`, `usvg`, `hayro_svg` (PDF→SVG), and `peniko`; it does **not** depend on `imaging`/`svg_imaging`.

### Cost & risk
This is the heavy phase: it reimplements the lowering `svg_imaging` already does (paint order, clip chains, masks, isolated groups, gradient baking, nested images) but against `KanvaSink`. Pin `usvg`/`hayro_svg` to the same versions `imaging` uses so parsing semantics match. Alternative considered and rejected: keep `svg_imaging` → `PaintSink` and merge consecutive fill+stroke passes on identical geometry in the builder - the same band-aid rejected for text/shapes, and it still can't emit kanva group/context labels for SVG sub-trees.

## Phasing

- **Phase 1 (the doubling fix):** `KanvaSink` trait + `KanvaBuilder` impl; `kanva_typst` with Group/Text/Shape/raster-Image; `typst_imaging` visibility changes; velyst switch. Delivers combined fill+stroke for text & shapes → `TraceFadeKanva` write-on works. SVG/PDF image items are skipped (logged) in this phase.
- **Phase 2 (`kanva_svg`):** native `usvg`/PDF walker → `KanvaSink` per the section above. Restores SVG/PDF content with combined fill+stroke and kanva-native group labels. Tracked separately from Phase 1.

## Verification

- `cargo build` across `kanva`, `kanva_typst`, `typst_imaging`, `velyst`, and `keyframes_vs_commands`.
- Unit test in `kanva` with a mock `KanvaSink` recorder: assert `draw_path`/`glyph_run` with both fill+stroke yield **one** `KanvaPath` (fill+stroke set), and path count == glyph count (not 2×).
- End-to-end: run `keyframes_vs_commands` with the label set to `TraceFadeKanva` (fill+stroke text); confirm the write-on traces stroke + fades fill on single paths, evenly across the timeline; optionally log `kanva.paths().len()` ≈ glyph count.
- Regression: static stroked text and stroked shapes render unchanged (modulo per-element vs all-fills-then-strokes paint order for overlapping geometry). SVG images: expect them missing in Phase 1 (acceptable for current PNG/text/shape content); restored in Phase 2.
