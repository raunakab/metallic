mod wgpu_bundle;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, Buffer, BufferUsages, Color, CommandEncoderDescriptor, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureViewDescriptor,
    VertexAttribute,
};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use crate::{
    primitives::{Point, PointFormat, Rect},
    rendering_engine::wgpu_bundle::{new_wgpu_bundle, WgpuBundle},
};

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

// // IoBundle
// // -----------------------------------------------------------------------------
// struct IoBundle {
//     relative_cursor_position: Option<PhysicalPosition<f32>>,
//     io_events: Vec<()>,
// }
// // -----------------------------------------------------------------------------

pub struct SceneBundle {
    pub background_color: Color,
    pub rects: Vec<Rect>,
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
        let scene_bundle = SceneBundle {
            background_color,
            rects: vec![],
        };
        Ok(Self {
            wgpu_bundle,
            scene_bundle,
        })
    }

    pub fn add_rect(&mut self, rect: Rect) {
        let size = self.wgpu_bundle.window.inner_size();
        let rect = rect.convert(PointFormat::Absolute, size);
        self.scene_bundle.rects.push(rect);
    }

    pub fn clear(&mut self) {
        self.scene_bundle.rects.clear();
    }

    pub fn redraw(&self) {
        self.wgpu_bundle.window.request_redraw();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.wgpu_bundle.surface_configuration.width = new_size.width;
        self.wgpu_bundle.surface_configuration.height = new_size.height;
        self.wgpu_bundle.surface.configure(&self.wgpu_bundle.device, &self.wgpu_bundle.surface_configuration);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
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
        .rects
        .iter()
        .flat_map(|rect| create_vertices(rect, size))
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

fn create_vertices(rect: &Rect, size: PhysicalSize<u32>) -> [Vertex; 6] {
    let Point { x: x1, y: y1, .. } = rect.tl.convert(PointFormat::Scaled, size);
    let Point { x: x2, y: y2, .. } = rect.br.convert(PointFormat::Scaled, size);
    let Color { r, g, b, a } = rect.color;
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
