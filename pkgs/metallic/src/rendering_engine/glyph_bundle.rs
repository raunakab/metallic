use std::path::Path;

use glyphon::{
    Buffer, Cache, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas, TextRenderer,
    Viewport,
};
use wgpu::{MultisampleState, RenderPass};
use winit::dpi::PhysicalSize;

use crate::{
    primitives::{convert_color, Text},
    rendering_engine::{wgpu_bundle::WgpuBundle, RenderingEngine},
    MetallicResult,
};

pub struct GlyphBundle {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub cache: Cache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
}

pub fn new_glyph_bundle(wgpu_bundle: &WgpuBundle) -> GlyphBundle {
    let font_system = FontSystem::new();
    let swash_cache = SwashCache::new();
    let cache = Cache::new(&wgpu_bundle.device);
    let viewport = Viewport::new(&wgpu_bundle.device, &cache);
    let mut atlas = TextAtlas::new(
        &wgpu_bundle.device,
        &wgpu_bundle.queue,
        &cache,
        wgpu_bundle.surface_configuration.format,
    );
    let text_renderer = TextRenderer::new(
        &mut atlas,
        &wgpu_bundle.device,
        MultisampleState::default(),
        None,
    );
    GlyphBundle {
        font_system,
        swash_cache,
        cache,
        viewport,
        atlas,
        text_renderer,
    }
}

pub fn prepare_text(
    rendering_engine: &mut RenderingEngine,
    size: PhysicalSize<u32>,
    text: Text,
) -> MetallicResult<()> {
    let mut buffer = Buffer::new(
        &mut rendering_engine.glyph_bundle.font_system,
        Metrics {
            font_size: text.font_size,
            line_height: text.line_height,
        },
    );
    buffer.set_size(
        &mut rendering_engine.glyph_bundle.font_system,
        size.width as _,
        size.height as _,
    );
    buffer.set_text(
        &mut rendering_engine.glyph_bundle.font_system,
        &text.text,
        text.attrs,
        text.shaping,
    );
    buffer.shape_until_scroll(&mut rendering_engine.glyph_bundle.font_system, text.prune);
    rendering_engine.glyph_bundle.text_renderer.prepare(
        &rendering_engine.wgpu_bundle.device,
        &rendering_engine.wgpu_bundle.queue,
        &mut rendering_engine.glyph_bundle.font_system,
        &mut rendering_engine.glyph_bundle.atlas,
        &rendering_engine.glyph_bundle.viewport,
        [TextArea {
            buffer: &buffer,
            top: text.top,
            left: text.left,
            scale: text.scale,
            bounds: text.bounds,
            default_color: convert_color(text.default_color),
        }],
        &mut rendering_engine.glyph_bundle.swash_cache,
    )?;
    Ok(())
}

pub fn draw_text<'a: 'b, 'b>(
    rendering_engine: &'a RenderingEngine,
    render_pass: &mut RenderPass<'b>,
) -> MetallicResult<()> {
    rendering_engine.glyph_bundle.text_renderer.render(
        &rendering_engine.glyph_bundle.atlas,
        &rendering_engine.glyph_bundle.viewport,
        render_pass,
    )?;
    Ok(())
}

pub fn resize_viewport(rendering_engine: &mut RenderingEngine, new_size: PhysicalSize<u32>) {
    rendering_engine.glyph_bundle.viewport.update(
        &rendering_engine.wgpu_bundle.queue,
        Resolution {
            width: new_size.width,
            height: new_size.height,
        },
    );
}

pub fn load_font<P: AsRef<Path>>(
    rendering_engine: &mut RenderingEngine,
    path: P,
) -> MetallicResult<()> {
    rendering_engine
        .glyph_bundle
        .font_system
        .db_mut()
        .load_font_file(path)?;
    Ok(())
}
