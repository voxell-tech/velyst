use std::sync::Arc;

use bevy::prelude::*;
use bevy_vello::{
    integrations::{VectorFile, VelloAsset},
    vello,
    vello_svg::{self, usvg},
    VelloScene,
};
use typst::{
    layout::{Abs, Frame, Page},
    model::Document,
};
use world::TypstWorld;

pub mod fonts;
pub mod world;

mod download;
mod package;

#[derive(Resource, Deref, DerefMut)]
pub struct TypstCompiler(Arc<TypstWorld>);

impl TypstCompiler {
    pub fn new(world: Arc<TypstWorld>) -> Self {
        Self(world)
    }
}

#[derive(Clone)]
pub struct TypstScene {
    pub scene: vello::Scene,
    pub width: f32,
    pub height: f32,
}

impl TypstScene {
    pub fn from_frame(frame: Frame) -> Result<Self, usvg::Error> {
        let svg_str = typst_svg::svg(&Page {
            frame,
            fill: typst::foundations::Smart::Auto,
            numbering: None,
            number: 0,
        });

        let tree = usvg::Tree::from_str(&svg_str, &usvg::Options::default())?;
        // print_ids(0, tree.root());

        // fn print_ids(indent: usize, group: &usvg::Group) {
        //     for node in group.children() {
        //         for _ in 0..indent * 2 {
        //             print!(" ");
        //         }

        //         match node {
        //             usvg::Node::Group(group) => {
        //                 println!("group: {}", group.id());
        //                 print_ids(indent + 1, group);
        //             }
        //             usvg::Node::Path(path) => println!("path: {}", path.id()),
        //             usvg::Node::Image(image) => println!("image: {}", image.id()),
        //             usvg::Node::Text(text) => println!("text: {}", text.id()),
        //         }
        //     }
        // }

        let scene = vello_svg::render_tree(&tree);
        let size = tree.size();

        Ok(Self {
            scene,
            width: size.width(),
            height: size.height(),
        })
    }

    pub fn from_document(document: &Document, padding: Abs) -> Result<Self, usvg::Error> {
        let svg_str = typst_svg::svg_merged(document, padding);

        println!("{}", svg_str);
        let tree = usvg::Tree::from_str(&svg_str, &usvg::Options::default())?;
        // print_ids(0, tree.root());

        // fn print_ids(indent: usize, group: &usvg::Group) {
        //     for node in group.children() {
        //         for _ in 0..indent * 2 {
        //             print!(" ");
        //         }

        //         match node {
        //             usvg::Node::Group(group) => {
        //                 println!("group: {}", group.id());
        //                 print_ids(indent + 1, group);
        //             }
        //             usvg::Node::Path(path) => println!("path: {}", path.id()),
        //             usvg::Node::Image(image) => println!("image: {}", image.id()),
        //             usvg::Node::Text(text) => println!("text: {}", text.id()),
        //         }
        //     }
        // }

        let scene = vello_svg::render_tree(&tree);
        let size = tree.size();

        Ok(Self {
            scene,
            width: size.width(),
            height: size.height(),
        })
    }

    pub fn as_asset(self) -> VelloAsset {
        let local_transform_center = Transform::from_xyz(self.width * 0.5, -self.height * 0.5, 0.0);

        VelloAsset {
            file: VectorFile::Svg(Arc::new(self.scene)),
            local_transform_center,
            width: self.width,
            height: self.height,
            alpha: 1.0,
        }
    }

    pub fn as_component(self) -> VelloScene {
        VelloScene::from(self.scene)
    }
}
