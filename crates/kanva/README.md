# Kanva

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/voxell-tech/velyst#license)
[![Crates.io](https://img.shields.io/crates/v/kanva.svg)](https://crates.io/crates/kanva)
[![Docs](https://docs.rs/kanva/badge.svg)](https://docs.rs/kanva/latest/kanva/)
[![CI](https://github.com/voxell-tech/velyst/workflows/CI/badge.svg)](https://github.com/voxell-tech/velyst/actions)

Backend-agnostic 2D graphics scene graph.

A `Kanva` stores vector graphics in a flat command buffer with grouping
support. Groups accumulate transforms onto child paths at render time without
mutating stored data. Overrides are expressed through `PathModifier` and
`GroupModifier` and cleared via `Kanva::clear_mods`.

Build a `Kanva` by feeding any `imaging::PaintSink` draw stream through
`KanvaBuilder`, then render it into any sink via `Kanva::render`.
Label nodes during build via `imaging::ContextRef` and look them up later
with `Kanva::query`, `Kanva::query_group`, or `Kanva::query_path`.

## Example

```rust
use kanva::builder::KanvaBuilder;
use kanva::prelude::*;
use kanva::imaging::kurbo::BezPath;
use kanva::imaging::peniko::Brush;
use kanva::imaging::record::Scene;
use kanva::imaging::{
    ContextRef, FillRef, GeometryRef, GroupRef, Composite, PaintSink,
};

// Build once from any imaging draw stream.
let mut builder = KanvaBuilder::new();
builder.push_context(ContextRef::new("wave", None));
builder.push_group(GroupRef::new());
let path = BezPath::new();
builder.fill(FillRef::new(GeometryRef::Path(&path), &Brush::default()));
builder.pop_group();
builder.pop_context();
let mut kanva = builder.build();

// Look up a labeled node.
let group_idx = kanva.query_group("wave").unwrap();

// Apply an override without mutating stored data.
kanva.set_group_mod(group_idx, GroupModifier {
    composite: Some(Composite::new(Default::default(), 0.5)),
    ..Default::default()
});

// Render into any PaintSink.
let mut scene = Scene::new();
kanva.render(&mut scene);
```

## Join the community!

You can join us on the [Voxell discord server](https://discord.gg/Mhnyp6VYEQ).

## License

`kanva` is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
