#[cfg(test)]
mod tests;

use bytemuck::{Pod, Zeroable};
use euclid::default::Point2D;
use lyon::path::Path;
use wgpu::{vertex_attr_array, Color, VertexAttribute};
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub point: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub const VERTEX_ATTRS: [VertexAttribute; 2] =
        vertex_attr_array![0 => Float32x2, 1 => Float32x4];
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AbsPoint(pub Point2D<f32>);

impl AbsPoint {
    pub fn to_scaled(&self, size: PhysicalSize<u32>) -> ScaledPoint {
        ScaledPoint(Point2D::new(
            abs_to_scaled_1d(self.0.x, size.width),
            -abs_to_scaled_1d(self.0.y, size.height),
        ))
    }
}

impl From<PhysicalPosition<f64>> for AbsPoint {
    fn from(physical_position: PhysicalPosition<f64>) -> Self {
        Self(Point2D::new(
            physical_position.x as _,
            physical_position.y as _,
        ))
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ScaledPoint(pub Point2D<f32>);

impl ScaledPoint {
    pub fn to_abs(&self, size: PhysicalSize<u32>) -> AbsPoint {
        AbsPoint(Point2D::new(
            scaled_to_abs_1d(self.0.x, size.width),
            scaled_to_abs_1d(-self.0.y, size.height),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Triangle(Triangle),
    Rect(Rect),
    Circle(Circle),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub a: AbsPoint,
    pub b: AbsPoint,
    pub c: AbsPoint,
    pub color: Color,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub tl: AbsPoint,
    pub br: AbsPoint,
    pub color: Color,
}

impl Rect {
    pub fn with_tl_and_size(tl: AbsPoint, color: Color, width: f32, height: f32) -> Self {
        let AbsPoint(Point2D { x, y, .. }) = tl;
        let br = AbsPoint(Point2D::new(x + width, y + height));
        Self { tl, br, color }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Circle;

fn abs_to_scaled_1d(a: f32, length: u32) -> f32 {
    (a / (length as f32)) * 2. - 1.
}
fn scaled_to_abs_1d(a: f32, length: u32) -> f32 {
    ((a + 1.0) / 2.0) * (length as f32)
}

pub fn point(x: f32, y: f32) -> AbsPoint {
    AbsPoint(Point2D::new(x, y))
}

#[cfg(test)]
fn scaled_point(x: f32, y: f32) -> ScaledPoint {
    ScaledPoint(Point2D::new(x, y))
}
