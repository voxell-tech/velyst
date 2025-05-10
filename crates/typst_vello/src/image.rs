use std::sync::Arc;

use typst::layout::{Size, Transform};
use typst::visualize as viz;
use vello::{kurbo, peniko};
use vello_svg::usvg;

use crate::utils::convert_transform;

#[derive(Default, Clone)]
pub struct ImageScene {
    pub transform: kurbo::Affine,
    pub scene: vello::Scene,
}

impl ImageScene {
    pub fn render(&self, scene: &mut vello::Scene) {
        scene.append(&self.scene, Some(self.transform));
    }
}

pub fn render_image(
    image: &viz::Image,
    size: Size,
    local_transform: Transform,
) -> ImageScene {
    // Size cannot be 0.
    debug_assert!(size.all(|p| p.to_pt() != 0.0));

    // TODO: The plan is to load it using bevy assets!

    match image.kind() {
        viz::ImageKind::Raster(raster) => {
            let mut scene = vello::Scene::new();

            let image = peniko::Image::new(
                peniko::Blob::new(Arc::new(
                    raster.dynamic().to_rgba8().into_vec(),
                )),
                peniko::ImageFormat::Rgba8,
                raster.width(),
                raster.height(),
            );

            scene.draw_image(&image, kurbo::Affine::IDENTITY);
            let (width, height) =
                (raster.width() as f64, raster.height() as f64);
            let transform = convert_transform(local_transform)
                .pre_scale_non_uniform(
                    size.x.to_pt() / width,
                    size.y.to_pt() / height,
                );

            ImageScene { transform, scene }
        }
        // TODO: Support paths in svg for animation.. (maybe a SvgScene?)
        viz::ImageKind::Svg(svg) => {
            let transform = convert_transform(local_transform)
                .pre_scale_non_uniform(
                    size.x.to_pt() / svg.width(),
                    size.y.to_pt() / svg.height(),
                );

            // FIXME: This is needed because the svg versions are different.
            let scene = match usvg::Tree::from_data(
                svg.data().as_slice(),
                &usvg::Options::default(),
            ) {
                Ok(tree) => vello_svg::render_tree(&tree),
                _ => vello::Scene::new(),
            };

            ImageScene { transform, scene }
        }
    }
}

impl std::fmt::Debug for ImageScene {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("FixedScene")
            .field("transform", &self.transform)
            .finish()
    }
}
