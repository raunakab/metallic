#[cfg(test)]
mod tests;

use bytemuck::{Pod, Zeroable};
use glyphon::{Attrs, Shaping, TextBounds};
use lyon::{
    math::{Point, Size},
    path::Path,
    tessellation::{FillVertex, FillVertexConstructor},
};
use wgpu::{vertex_attr_array, Color, VertexAttribute};
use winit::dpi::PhysicalSize;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub topleft: Point,
    pub size: Size,
    pub bounds: TextBounds,
    pub scale: f32,
    pub attrs: Attrs<'static>,
    pub shaping: Shaping,
    pub color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::default(),
            font_size: 30.0,
            line_height: 30.0,
            topleft: Point::zero(),
            size: Size::zero(),
            bounds: TextBounds {
                top: 0,
                left: 0,
                bottom: i32::MAX,
                right: i32::MAX,
            },
            scale: 1.0,
            attrs: Attrs::new(),
            shaping: Shaping::Basic,
            color: Color::BLACK,
        }
    }
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

impl FillVertexConstructor<Point> for Ctor {
    fn new_vertex(&mut self, vertex: FillVertex) -> Point {
        vertex.position()
    }
}

pub(crate) fn to_vertex(point_2d: Point, size: PhysicalSize<u32>, color: Color) -> Vertex {
    // let x = abs_to_scaled_1d(point_2d.x, size.width);
    // let y = -abs_to_scaled_1d(point_2d.y, size.height);
    // let Color { r, g, b, a } = color;
    // Vertex {
    //     point: [x, y],
    //     color: [r as _, g as _, b as _, a as _],
    // }
    todo!()
}

// fn abs_to_scaled_1d(x: f32, length: u32) -> f32 {
//     (x / (length as f32)) * 2. - 1.
// }
// fn f64_to_u8(a: f64) -> u8 {
//     (a * (u8::MAX as f64)) as _
// }
// pub(crate) fn convert_color(Color { r, g, b, a }: Color) -> GColor {
//     let r = f64_to_u8(r);
//     let g = f64_to_u8(g);
//     let b = f64_to_u8(b);
//     let a = f64_to_u8(a);
//     GColor::rgba(r, g, b, a)
// }
