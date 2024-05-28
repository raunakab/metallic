use bytemuck::{Pod, Zeroable};
use euclid::default::Point2D;
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

#[cfg(test)]
mod tests {
    use super::*;

    const LENGTH: u32 = 100;

    #[test]
    fn test_abs_to_scaled_conversion() {
        let inputs = [
            (0.0, LENGTH),
            ((LENGTH / 2) as _, LENGTH),
            (LENGTH as _, LENGTH),
        ];
        let expected_outputs = [-1.0, 0.0, 1.0];
        assert_eq!(inputs.len(), expected_outputs.len());
        for ((x, length), expected_output) in inputs.into_iter().zip(expected_outputs) {
            let actual_output = abs_to_scaled_1d(x, length);
            assert_eq!(actual_output, expected_output);
        }
    }
}
