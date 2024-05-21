mod wgpu_bundle;

use std::collections::VecDeque;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, Buffer, BufferUsages, Color, CommandEncoderDescriptor, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureViewDescriptor,
    VertexAttribute,
};
use winit::{dpi::PhysicalSize, event::ElementState, event_loop::ActiveEventLoop};

use crate::{
    primitives::{IoEvent, MouseInput, Point, PointFormat, Rect, Shape, ShapeType},
    rendering_engine::wgpu_bundle::{new_wgpu_bundle, WgpuBundle},
};

const IO_EVENTS_CAPACITY: usize = 8;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    point: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    pub const VERTEX_ATTRS: [VertexAttribute; 2] =
        vertex_attr_array![0 => Float32x2, 1 => Float32x4];
}

struct IoBundle {
    io_events: VecDeque<IoEvent>,
    cursor_position: Option<Point>,
}

pub struct SceneBundle {
    pub background_color: Color,
    pub shapes: Vec<Shape>,
}

pub struct RenderingEngine {
    wgpu_bundle: WgpuBundle,
    scene_bundle: SceneBundle,
    io_bundle: IoBundle,
}

impl RenderingEngine {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        background_color: Color,
    ) -> anyhow::Result<Self> {
        let wgpu_bundle = new_wgpu_bundle(event_loop).await?;
        Ok(Self {
            wgpu_bundle,
            scene_bundle: SceneBundle {
                background_color,
                shapes: vec![],
            },
            io_bundle: IoBundle {
                io_events: VecDeque::with_capacity(IO_EVENTS_CAPACITY),
                cursor_position: None,
            },
        })
    }

    pub fn register_io_event(&mut self, io_event: IoEvent) {
        while self.io_bundle.io_events.len() >= IO_EVENTS_CAPACITY {
            self.io_bundle.io_events.pop_front();
        }
        self.io_bundle.io_events.push_back(io_event);
    }

    fn handle_io_events(&mut self) {
        while let Some(io_event) = self.io_bundle.io_events.pop_front() {
            match io_event {
                IoEvent::MouseInput(MouseInput { state, .. }) => {
                    if let Some(point) = self.io_bundle.cursor_position {
                        let point = point
                            .convert(PointFormat::Absolute, self.wgpu_bundle.window.inner_size());
                        let msg = match state {
                            ElementState::Pressed => "Pressed",
                            ElementState::Released => "Released",
                        };
                        println!("{} at location: {:?}", msg, point);
                    };
                }
                IoEvent::CursorMoved(new_point) => self.io_bundle.cursor_position = Some(new_point),
            }
        }
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.scene_bundle.shapes.push(shape);
    }

    pub fn clear(&mut self) {
        self.scene_bundle.shapes.clear();
    }

    pub fn redraw(&self) {
        self.wgpu_bundle.window.request_redraw();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.wgpu_bundle.surface_configuration.width = new_size.width;
        self.wgpu_bundle.surface_configuration.height = new_size.height;
        self.wgpu_bundle.surface.configure(
            &self.wgpu_bundle.device,
            &self.wgpu_bundle.surface_configuration,
        );
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.handle_io_events();
        let (buffer, num_of_vertices) = create_buffer(self);
        let surface_texture = self.wgpu_bundle.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .wgpu_bundle
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.scene_bundle.background_color),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.wgpu_bundle.render_pipeline);
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            render_pass.draw(0..(num_of_vertices as _), 0..1);
        };
        let command_buffer = encoder.finish();
        self.wgpu_bundle.queue.submit([command_buffer]);
        surface_texture.present();
        Ok(())
    }
}

fn create_buffer(rendering_engine: &RenderingEngine) -> (Buffer, usize) {
    let size = rendering_engine.wgpu_bundle.window.inner_size();
    let vertices = rendering_engine
        .scene_bundle
        .shapes
        .iter()
        .flat_map(|&shape| create_vertices(shape, size))
        .collect::<Vec<_>>();
    let num_of_vertices = vertices.len();
    let buffer = rendering_engine
        .wgpu_bundle
        .device
        .create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &cast_slice(&*vertices),
            usage: BufferUsages::VERTEX,
        });
    (buffer, num_of_vertices)
}

fn create_vertices(shape: Shape, size: PhysicalSize<u32>) -> [Vertex; 6] {
    match shape.shape_type {
        ShapeType::Rect(rect) => {
            let Point { x: x1, y: y1, .. } = rect.tl.convert(PointFormat::Scaled, size);
            let Point { x: x2, y: y2, .. } = rect.br.convert(PointFormat::Scaled, size);
            let Color { r, g, b, a } = shape.properties.color;
            let color = [r as _, g as _, b as _, a as _];
            let tl = Vertex {
                point: [x1, y1],
                color,
            };
            let tr = Vertex {
                point: [x2, y1],
                color,
            };
            let bl = Vertex {
                point: [x1, y2],
                color,
            };
            let br = Vertex {
                point: [x2, y2],
                color,
            };
            [tl, br, tr, tl, bl, br]
        }
    }
}
