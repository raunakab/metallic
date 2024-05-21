#[cfg(test)]
mod tests;

use wgpu::Color;
use winit::dpi::PhysicalSize;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointFormat {
    Absolute,
    Scaled,
}

pub struct Rect {
    pub tl: Point,
    pub br: Point,
    pub color: Color,
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
