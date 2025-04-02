// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{ExampleScene, SceneConfig, SceneParams, SceneSet};
use vello::kurbo::Affine;
use vello::*;

macro_rules! scene {
    ($name: ident) => {
        scene!($name: false)
    };
    ($name: ident: animated) => {
        scene!($name: true)
    };
    ($name: ident: $animated: literal) => {
        scene!($name, stringify!($name), $animated)
    };
    ($func:expr, $name: expr, $animated: literal) => {
        ExampleScene {
            config: SceneConfig {
                animated: $animated,
                name: $name.to_owned(),
            },
            function: Box::new($func),
        }
    };
}

pub fn test_scenes() -> SceneSet {
    let scenes = vec![scene!(splash_with_tiger(), "Tiger", true)];
    SceneSet { scenes }
}

// Scenes
fn splash_screen(scene: &mut Scene, params: &mut SceneParams) {
    let strings = [
        "Vello SVG Demo",
        #[cfg(not(target_arch = "wasm32"))]
        "  Arrow keys: switch scenes",
        "  Space: reset transform",
        "  S: toggle stats",
        "  V: toggle vsync",
        "  M: cycle AA method",
        "  Q, E: rotate",
    ];
    // Tweak to make it fit with tiger
    let a = Affine::scale(0.11) * Affine::translate((-90.0, -50.0));
    for (i, s) in strings.iter().enumerate() {
        let text_size = if i == 0 { 60.0 } else { 40.0 };
        params.text.add(
            scene,
            None,
            text_size,
            None,
            a * Affine::translate((100.0, 100.0 + 60.0 * i as f64)),
            s,
        );
    }
}

fn splash_with_tiger() -> impl FnMut(&mut Scene, &mut SceneParams) {
    let contents = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../assets/Ghostscript_Tiger.svg"
    ));
    let mut tiger = crate::svg::svg_function_of("Ghostscript Tiger".to_string(), move || contents);
    move |scene, params| {
        tiger(scene, params);
        splash_screen(scene, params);
    }
}
