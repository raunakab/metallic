#[cfg(test)]
mod tests;

use bytemuck::{Pod, Zeroable};
use euclid::default::Point2D;
use glyphon::{Attrs, Color as GlyphonColor, Shaping, TextBounds};
use lyon::{
    path::Path,
    tessellation::{FillVertex, FillVertexConstructor},
};
use wgpu::{vertex_attr_array, Color, VertexAttribute};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct Vertex {
    pub point: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub(crate) const VERTEX_ATTRS: [VertexAttribute; 2] =
        vertex_attr_array![0 => Float32x2, 1 => Float32x4];
}

pub enum Object {
    Shape(Shape),
    Text(Text),
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub path: Path,
    pub color: Color,
}

pub(crate) struct Ctor;

impl FillVertexConstructor<Point2D<f32>> for Ctor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Point2D<f32> {
        vertex.position()
    }
}

pub(crate) fn to_vertex(point_2d: Point2D<f32>, size: PhysicalSize<u32>, color: Color) -> Vertex {
    let x = abs_to_scaled_1d(point_2d.x, size.width);
    let y = -abs_to_scaled_1d(point_2d.y, size.height);
    let Color { r, g, b, a } = color;
    Vertex {
        point: [x, y],
        color: [r as _, g as _, b as _, a as _],
    }
}

fn abs_to_scaled_1d(x: f32, length: u32) -> f32 {
    (x / (length as f32)) * 2. - 1.
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub text: String,
    pub attrs: Attrs<'static>,
    pub shaping: Shaping,
    pub prune: bool,
    pub line_height: f32,
    pub font_size: f32,
    pub top: f32,
    pub left: f32,
    pub scale: f32,
    pub bounds: TextBounds,
    pub default_color: Color,
}

pub(crate) fn convert_color(Color { r, g, b, a }: Color) -> GlyphonColor {
    fn f64_to_u8(x: f64) -> u8 {
        (x * (u8::MAX as f64)) as _
    }
    let r = f64_to_u8(r);
    let g = f64_to_u8(g);
    let b = f64_to_u8(b);
    let a = f64_to_u8(a);
    GlyphonColor::rgba(r, g, b, a)
}
