# Kanva - 2D Graphics Scene Graph

## Overview

Kanva is a backend-agnostic 2D graphics scene graph library. It stores vector graphics in a flat command buffer with grouping support for accumulated transform animation.

**Relationship with motiongfx**: Kanva provides the scene graph structure and exposes animatable fields via `PathModifier` / `GroupModifier`. motiongfx handles the animation/timeline layer.

## Typst Integration

Typst provides dynamic content (re-renders on changes). `KanvaBuilder` implements `PaintSink` to receive Typst's imaging output and bake it into a `Kanva`. motiongfx then animates the resulting kanva's fields frame-by-frame.

## Data Structures

```rust
struct Kanva {
    commands:   Vec<Command>,
    groups:     Vec<Group>,
    group_ends: Vec<usize>,               // parallel to groups: command index of PopGroup
    paths:      Vec<KanvaPath>,
    index:      HashMap<Box<str>, NodeIndex>, // label -> Group(idx) | Path(idx)
    path_mods:  HashMap<usize, PathModifier>,
    group_mods: HashMap<usize, GroupModifier>,
}

struct Group {
    transform: Affine,         // animation delta; accumulated in render
    composite: Composite,
    clip: Option<KanvaClip>,   // inline clip (not a path index)
}

struct KanvaClip {
    path:      BezPath,
    transform: Affine,
    style:     peniko::Style,  // Fill(rule) | Stroke(stroke)
}

struct KanvaPath {
    path:      BezPath,
    transform: Affine,    // full world transform as received from imaging
    fill:      Option<KanvaFill>,
    stroke:    Option<KanvaStroke>,
}

struct KanvaFill {
    rule:            Fill,
    brush:           Brush,
    brush_transform: Option<Affine>,
    composite:       Composite,
}

struct KanvaStroke {
    stroke:          Stroke,
    brush:           Brush,
    brush_transform: Option<Affine>,
    composite:       Composite,
}

enum Command {
    PushGroup(usize),  // index into groups
    PopGroup,
    DrawPath(usize),   // index into paths
}

enum NodeIndex {
    Group(usize),
    Path(usize),
}
```

## Modifiers

Modifiers are applied per-frame at render time and never mutate primary data. Clearing them (via `clear_mods()`) restores original appearance without a rebuild.

```rust
struct PathModifier {
    path:      Option<BezPath>,   // replaces path.path
    fill:      Option<KanvaFill>, // replaces path.fill
    stroke:    Option<KanvaStroke>,
    alpha:     Option<f32>,       // wraps draws in an isolated group
    transform: Option<Affine>,    // replaces path.transform before group deltas applied
}

struct GroupModifier {
    transform: Option<Affine>,    // replaces group.transform
    clip:      Option<KanvaClip>,
    composite: Option<Composite>,
}
```

Usage:
```rust
kanva.set_path_mod(idx, PathModifier { alpha: Some(0.5), ..default() });
kanva.set_group_mod(idx, GroupModifier { transform: Some(delta), ..default() });
kanva.clear_mods(); // revert all overrides
```

## Building from Imaging

`KanvaBuilder` implements `PaintSink`. Feed it any imaging draw stream to produce a `Kanva`:

```rust
let mut builder = KanvaBuilder::new();
// pass builder to any imaging renderer / Typst output:
typst_scene.render(&mut builder);
let kanva = builder.build();
```

`push_context` / `pop_context` calls label the next group or path for later lookup:
```rust
kanva.query("shape_label") // -> Option<NodeIndex>
```

## Rendering

`Kanva::render` walks `commands` with an `Affine` stack to accumulate group animation deltas:

```rust
kanva.render(&mut sink); // sink: impl PaintSink
```

- `PushGroup`: multiplies the group's (possibly overridden) transform onto the stack; pushes a `GroupRef` to the sink with resolved clip + composite.
- `PopGroup`: pops the stack; calls `sink.pop_group()`.
- `DrawPath`: applies `group_tf * base_tf`; if `alpha` modifier is set, wraps draws in an isolated group.

`group_ends[i]` is the command index of group `i`'s `PopGroup` command — useful for subtree skipping in future work.

## Key Decisions

- **Everything is a `BezPath`** — no glyphs, no images; callers outline text before passing to kanva.
- **`path.transform` is the full world transform** stored as-is from imaging. Group transforms are animation deltas on top.
- **Modifiers never touch primary data** — primary fields (`path`, `transform`, `fill`, `stroke`) are write-once at build time.
- **No glyphs** — keeps kanva simple; users use `ttf-parser` / `fontcore` to outline glyphs externally.

## Status

- [x] Core types: `Kanva`, `Group`, `KanvaClip`, `KanvaPath`, `KanvaFill`, `KanvaStroke`
- [x] Command buffer + `group_ends`
- [x] `PathModifier` + `GroupModifier`
- [x] `KanvaBuilder` (`PaintSink` impl)
- [x] `Kanva::render` with transform accumulation and modifier support
- [x] Tests: 18 passing (8 builder, 10 render)
