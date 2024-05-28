use std::path::Path;

use glyphon::{
    Buffer, Cache, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas, TextRenderer,
    Viewport,
};
use wgpu::{MultisampleState, RenderPass};
use winit::dpi::PhysicalSize;

use crate::{
    primitives::{convert_color, Text},
    rendering_engine::RenderingEngine,
    MetallicResult,
};

pub struct GlyphBundle {
    pub font_system: FontSystem,
}

pub struct PreparedTextBundle {
    pub text_renderer: TextRenderer,
    pub atlas: TextAtlas,
    pub viewport: Viewport,
}

pub fn prepare_text(
    rendering_engine: &mut RenderingEngine,
    size: PhysicalSize<u32>,
    text: Text,
    depth: usize,
) -> MetallicResult<PreparedTextBundle> {
    let mut swash_cache = SwashCache::new();
    let cache = Cache::new(&rendering_engine.wgpu_bundle.device);
    let mut viewport = Viewport::new(&rendering_engine.wgpu_bundle.device, &cache);
    let mut atlas = TextAtlas::new(
        &rendering_engine.wgpu_bundle.device,
        &rendering_engine.wgpu_bundle.queue,
        &cache,
        rendering_engine.wgpu_bundle.surface_configuration.format,
    );
    let mut text_renderer = TextRenderer::new(
        &mut atlas,
        &rendering_engine.wgpu_bundle.device,
        MultisampleState::default(),
        None,
    );
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
    viewport.update(
        &rendering_engine.wgpu_bundle.queue,
        Resolution {
            width: size.width,
            height: size.height,
        },
    );
    text_renderer.prepare_with_depth(
        &rendering_engine.wgpu_bundle.device,
        &rendering_engine.wgpu_bundle.queue,
        &mut rendering_engine.glyph_bundle.font_system,
        &mut atlas,
        &viewport,
        [TextArea {
            buffer: &buffer,
            top: text.top,
            left: text.left,
            scale: text.scale,
            bounds: text.bounds,
            default_color: convert_color(text.default_color),
        }],
        &mut swash_cache,
        |_| depth as _,
    )?;
    Ok(PreparedTextBundle {
        text_renderer,
        atlas,
        viewport,
    })
}

pub fn draw_text<'b>(
    prepared_text_bundle: &'b PreparedTextBundle,
    render_pass: &mut RenderPass<'b>,
) -> MetallicResult<()> {
    // prepared_text_bundle.text_renderer.prepare_with_depth(, , , , , , , )
    prepared_text_bundle.text_renderer.render(
        &prepared_text_bundle.atlas,
        &prepared_text_bundle.viewport,
        render_pass,
    )?;
    Ok(())
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
