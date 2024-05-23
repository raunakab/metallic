mod wgpu_bundle;

use bytemuck::cast_slice;
use hashbrown::HashMap;
use uuid::Uuid;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureViewDescriptor,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::ActiveEventLoop,
};

use crate::{
    primitives::{ScaledPoint, Shape, ShapeType, Vertex},
    rendering_engine::wgpu_bundle::{new_wgpu_bundle, WgpuBundle},
};

pub struct SceneBundle {
    pub background_color: Color,
    pub shapes: HashMap<Uuid, Shape>,
}

pub struct RenderingEngine {
    wgpu_bundle: WgpuBundle,
    scene_bundle: SceneBundle,
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
                shapes: HashMap::default(),
            },
        })
    }

    pub fn add_shape(&mut self, shape: Shape) {
        let id = Uuid::new_v4();
        self.scene_bundle.shapes.insert(id, shape);
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
        let buffer_bundle = create_buffer_bundle(self);
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
            render_pass.set_vertex_buffer(0, buffer_bundle.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffer_bundle.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..(buffer_bundle.index_buffer_size as _), 0, 0..1);
        };
        let command_buffer = encoder.finish();
        self.wgpu_bundle.queue.submit([command_buffer]);
        surface_texture.present();
        Ok(())
    }
}

struct BufferBundle {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_buffer_size: usize,
}

fn create_buffer_bundle(rendering_engine: &RenderingEngine) -> BufferBundle {
    let size = rendering_engine.wgpu_bundle.window.inner_size();
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut offset = 0;
    for (_, shape) in &rendering_engine.scene_bundle.shapes {
        match shape.shape_type {
            ShapeType::Rect(rect) => {
                let ScaledPoint(PhysicalPosition { x: x1, y: y1 }) = rect.tl.to_scaled(size);
                let ScaledPoint(PhysicalPosition { x: x2, y: y2 }) = rect.br.to_scaled(size);
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
                vertices.extend([tl, tr, bl, br]);
                indices.extend([
                    offset,
                    offset + 3,
                    offset + 1,
                    offset,
                    offset + 2,
                    offset + 3,
                ]);
                offset += 4;
            }
        }
    }
    let vertex_buffer =
        rendering_engine
            .wgpu_bundle
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });
    let index_buffer =
        rendering_engine
            .wgpu_bundle
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &cast_slice::<u16, _>(&indices),
                usage: BufferUsages::INDEX,
            });
    BufferBundle {
        vertex_buffer,
        index_buffer,
        index_buffer_size: indices.len(),
    }
}
