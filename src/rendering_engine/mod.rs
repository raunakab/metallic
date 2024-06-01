mod bundles;

use std::path::Path;

use lyon::math::Size;
use wgpu::Color;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

use crate::{
    primitives::Object,
    rendering_engine::bundles::{
        create_scene_bundle, create_wgpu_bundle, load_font, redraw, render, resize, SceneBundle,
        WgpuBundle,
    },
    MetallicResult,
};

pub struct RenderingEngine {
    wgpu_bundle: WgpuBundle,
    scene_bundle: SceneBundle,
}

impl RenderingEngine {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        background_color: Color,
    ) -> MetallicResult<Self> {
        let wgpu_bundle = create_wgpu_bundle(event_loop).await?;
        let scene_bundle = create_scene_bundle(background_color);
        Ok(Self {
            wgpu_bundle,
            scene_bundle,
        })
    }

    pub fn load_font(&mut self, path: impl AsRef<Path>) -> MetallicResult<()> {
        load_font(self, path)
    }

    pub fn object_layers(&mut self) -> &mut Vec<Vec<Object>> {
        &mut self.scene_bundle.object_layers
    }

    pub fn size(&self) -> Size {
        let size = self.wgpu_bundle.window.inner_size();
        Size::new(size.width as _, size.height as _)
    }

    pub fn redraw(&self) {
        redraw(self);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        resize(self, new_size)
    }

    pub fn render(&mut self) -> MetallicResult<()> {
        render(self)
    }
}
