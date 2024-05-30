#[cfg(test)]
mod tests;

use bytemuck::{Pod, Zeroable};
use cosmic_text::Color as CTColor;
use euclid::default::Point2D;
use lyon::{
    path::Path,
    tessellation::{FillVertex, FillVertexConstructor},
};
use wgpu::{vertex_attr_array, Color, VertexAttribute};
use winit::dpi::PhysicalSize;

#[derive(Default, Debug, Clone)]
pub struct LayeredObject {
    pub layer: usize,
    pub object: Object,
}

#[derive(Debug, Clone)]
pub enum Object {
    Shape(Shape),
    Text(Text),
}

impl Default for Object {
    fn default() -> Self {
        Self::Shape(Shape::default())
    }
}

#[derive(Default, Debug, Clone)]
pub struct Shape {
    pub path: Path,
    pub color: Color,
}

#[derive(Default, Debug, Clone)]
pub struct Text {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
}

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

pub(crate) fn color_converter(Color { r, g, b, a }: Color) -> CTColor {
    fn convert(x: f64) -> u8 { todo!() }
    let r = convert(r);
    let g = convert(g);
    let b = convert(b);
    let a = convert(a);
    CTColor::rgba(r, g, b, a)
}
