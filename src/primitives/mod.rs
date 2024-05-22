#[cfg(test)]
mod tests;

use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, Color, VertexAttribute};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IoEvent {
    MouseInput(MouseInput),
    CursorMoved(AbsPoint),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AbsPoint(pub PhysicalPosition<f32>);

impl AbsPoint {
    pub fn to_scaled(self, size: PhysicalSize<u32>) -> ScaledPoint {
        ScaledPoint(PhysicalPosition {
            x: abs_to_scaled_1d(self.0.x, size.width),
            y: -abs_to_scaled_1d(self.0.y, size.height),
        })
    }
}

impl From<PhysicalPosition<f64>> for AbsPoint {
    fn from(point: PhysicalPosition<f64>) -> Self {
        Self(PhysicalPosition {
            x: point.x as _,
            y: point.y as _,
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ScaledPoint(pub PhysicalPosition<f32>);

impl ScaledPoint {
    pub fn to_abs(self, size: PhysicalSize<u32>) -> AbsPoint {
        AbsPoint(PhysicalPosition {
            x: scaled_to_abs_1d(self.0.x, size.width),
            y: scaled_to_abs_1d(-self.0.y, size.height),
        })
    }
}

#[derive(Clone)]
pub struct Shape {
    pub properties: Properties,
    pub shape_type: ShapeType,
}

impl Shape {
    pub fn contains_point(&self, abs_point: AbsPoint) -> bool {
        match self.shape_type {
            ShapeType::Rect(rect) => {
                let AbsPoint(PhysicalPosition { x, y }) = abs_point;
                let AbsPoint(PhysicalPosition { x: x1, y: y1 }) = rect.tl;
                let AbsPoint(PhysicalPosition { x: x2, y: y2 }) = rect.br;
                x1 <= x && x <= x2 && y1 <= y && y <= y2
            }
        }
    }

    pub fn click(&self, mouse_input: MouseInput) {
        if let Some(on_io_event) = self.properties.on_mouse_input.as_ref() {
            on_io_event(mouse_input);
        }
    }
}

#[derive(Default, Clone)]
pub struct Properties {
    pub color: Color,
    pub on_mouse_input: Option<Rc<dyn Fn(MouseInput)>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    Rect(Rect),
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Rect {
    pub tl: AbsPoint,
    pub br: AbsPoint,
}

fn abs_to_scaled_1d(a: f32, length: u32) -> f32 {
    (a / (length as f32)) * 2. - 1.
}
fn scaled_to_abs_1d(a: f32, length: u32) -> f32 {
    ((a + 1.0) / 2.0) * (length as f32)
}
