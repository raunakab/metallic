use glyphon::{Attrs, Shaping, TextBounds, Color as GColor};
use lyon::{
    math::{Point, Size},
    path::Path,
};
use wgpu::Color;

#[derive(Default, Debug, Clone)]
pub struct Object {
    pub(crate) object_kind: ObjectKind,
    pub(crate) brush: Brush,
}

#[derive(Debug, Clone)]
pub enum ObjectKind {
    Shape(Path),
    Text(Text),
}

impl Default for ObjectKind {
    fn default() -> Self {
        Self::Shape(Path::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Brush {
    Solid(Color),
}

impl Default for Brush {
    fn default() -> Self {
        Self::Solid(Color::BLACK)
    }
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

pub fn shape(path: Path, brush: Brush) -> Object {
    Object {
        object_kind: ObjectKind::Shape(path),
        brush,
    }
}

pub fn text(text: Text, brush: Brush) -> Object {
    Object {
        object_kind: ObjectKind::Text(text),
        brush,
    }
}

pub(crate) fn convert_color(Color { r, g, b, a }: Color) -> GColor {
    fn convert(x: f64) -> u8 {
        (x * (u8::MAX as f64)) as u8
    }
    let r = convert(r);
    let g = convert(g);
    let b = convert(b);
    let a = convert(a);
    GColor::rgba(r, g, b, a)
}
