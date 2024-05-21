#[cfg(test)]
mod tests;

use wgpu::Color;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IoEvent {
    MouseInput(MouseInput),
    CursorMoved(Point),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub point_format: PointFormat,
}

impl Point {
    pub fn convert(self, new_point_format: PointFormat, size: PhysicalSize<u32>) -> Self {
        match (self.point_format, new_point_format) {
            (PointFormat::Absolute, PointFormat::Scaled) => Self {
                x: abs_to_scaled(self.x, size.width),
                y: -abs_to_scaled(self.y, size.height),
                point_format: new_point_format,
            },
            (PointFormat::Scaled, PointFormat::Absolute) => Self {
                x: scaled_to_abs(self.x, size.width),
                y: scaled_to_abs(-self.y, size.height),
                point_format: new_point_format,
            },
            _ => self,
        }
    }
}

impl From<PhysicalPosition<f64>> for Point {
    fn from(position: PhysicalPosition<f64>) -> Self {
        Self {
            x: position.x as _,
            y: position.y as _,
            point_format: PointFormat::Absolute,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointFormat {
    Absolute,
    Scaled,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Shape {
    pub properties: Properties,
    pub shape_type: ShapeType,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Properties {
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    Rect(Rect),
}

#[derive(Clone, Copy, PartialEq)]
pub struct Rect {
    pub tl: Point,
    pub br: Point,
}

impl Rect {
    pub fn convert(self, new_point_format: PointFormat, size: PhysicalSize<u32>) -> Self {
        Self {
            tl: self.tl.convert(new_point_format, size),
            br: self.br.convert(new_point_format, size),
            ..self
        }
    }
}

fn abs_to_scaled(a: f32, length: u32) -> f32 {
    (a / (length as f32)) * 2. - 1.
}

fn scaled_to_abs(a: f32, length: u32) -> f32 {
    ((a + 1.0) / 2.0) * (length as f32)
}
