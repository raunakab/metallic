mod wgpu_bundle;

use bytemuck::cast_slice;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, VertexBuffers};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use crate::{
    primitives::{to_vertex, Ctor, Shape, Vertex},
    rendering_engine::wgpu_bundle::{new_wgpu_bundle, WgpuBundle},
    MetallicResult,
};

pub struct SceneBundle {
    background_color: Color,
    shapes: Vec<(Shape, usize)>,
    layer: usize,
    fill_tessellator: FillTessellator,
}

pub struct RenderingEngine {
    wgpu_bundle: WgpuBundle,
    scene_bundle: SceneBundle,
}

impl RenderingEngine {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        background_color: Color,
    ) -> MetallicResult<Self> {
        let wgpu_bundle = new_wgpu_bundle(event_loop).await?;
        Ok(Self {
            wgpu_bundle,
            scene_bundle: SceneBundle {
                background_color,
                shapes: vec![],
                layer: 0,
                fill_tessellator: FillTessellator::default(),
            },
        })
    }

    pub fn push_layer(&mut self) {
        self.scene_bundle.layer = self
            .scene_bundle
            .layer
            .checked_add(1)
            .expect("Too many layers pushed");
    }

    pub fn pop_layer(&mut self) {
        self.scene_bundle.layer = self.scene_bundle.layer.saturating_sub(1);
    }

    pub fn add_shape(&mut self, shape: Shape) {
        let layer = self.scene_bundle.layer;
        let index = match self
            .scene_bundle
            .shapes
            .binary_search_by(|&(_, curr_layer)| curr_layer.cmp(&layer))
        {
            Ok(index) => index + 1,
            Err(index) => index,
        };
        self.scene_bundle.shapes.insert(index, (shape, layer));
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

    pub fn render(&mut self) -> MetallicResult<()> {
        let buffer_bundle = create_buffer_bundle(self)?;
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

fn create_buffer_bundle(rendering_engine: &mut RenderingEngine) -> MetallicResult<BufferBundle> {
    let size = rendering_engine.wgpu_bundle.window.inner_size();
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut offset = 0;
    for (shape, _) in &rendering_engine.scene_bundle.shapes {
        let mut geometry = VertexBuffers::<_, u16>::new();
        let mut buffers_builder = BuffersBuilder::new(&mut geometry, Ctor);
        rendering_engine
            .scene_bundle
            .fill_tessellator
            .tessellate_path(
                &shape.path,
                &FillOptions::tolerance(0.02),
                &mut buffers_builder,
            )?;
        let length = geometry.vertices.len();
        vertices.extend(
            geometry
                .vertices
                .into_iter()
                .map(|point_2d| to_vertex(point_2d, size, shape.color)),
        );
        indices.extend(geometry.indices.into_iter().map(|index| index + offset));
        offset += length as u16;
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
                contents: &cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });
    Ok(BufferBundle {
        vertex_buffer,
        index_buffer,
        index_buffer_size: indices.len(),
    })
}
