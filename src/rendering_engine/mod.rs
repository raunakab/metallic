mod bundles;

use std::path::Path;

use wgpu::{
    Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use crate::{
    primitives::{LayeredObject, Object},
    rendering_engine::bundles::*,
    MetallicResult,
};

pub struct RenderingEngine {
    wgpu_bundle: WgpuBundle,
    scene_bundle: SceneBundle,
    glyph_bundle: GlyphBundle,
}

impl RenderingEngine {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        background_color: Color,
    ) -> MetallicResult<Self> {
        let wgpu_bundle = create_wgpu_bundle(event_loop).await?;
        Ok(Self {
            wgpu_bundle,
            scene_bundle: create_scene_bundle(background_color),
            glyph_bundle: create_glyph_bundle(),
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

    pub fn load_font(&mut self, path: impl AsRef<Path>) -> MetallicResult<()> {
        self.glyph_bundle
            .font_system
            .db_mut()
            .load_font_file(path)?;
        Ok(())
    }

    pub fn add_object(&mut self, object: Object) {
        let layer = self.scene_bundle.layer;
        let index = match self
            .scene_bundle
            .layered_objects
            .binary_search_by(|layered_object| layered_object.layer.cmp(&layer))
        {
            Ok(index) => index + 1,
            Err(index) => index,
        };
        self.scene_bundle
            .layered_objects
            .insert(index, LayeredObject { layer, object });
    }

    pub fn clear(&mut self) {
        self.scene_bundle.layered_objects.clear();
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
        let frame = self.wgpu_bundle.surface.get_current_texture()?;
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .wgpu_bundle
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        let buffer_bundle = create_buffer_bundle(self)?;
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
        frame.present();
        Ok(())
    }
}
