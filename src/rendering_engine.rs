use std::{collections::BTreeMap, path::Path};

use glyphon::FontSystem;
use wgpu::{
    Color, CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{primitives::Object, InvalidConfigurationError, MetallicError, MetallicResult};

// Rendering Engine
// =============================================================================

pub struct RenderingEngine {
    #[allow(dead_code)]
    instance: Instance,
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    surface_configuration: SurfaceConfiguration,
    background_color: Color,
    object_engine: ObjectEngine,
    font_system: FontSystem,
}

impl Drop for RenderingEngine {
    fn drop(&mut self) {
        let window = self.window as *const _ as *mut Window;
        let _ = unsafe { Box::from_raw(window) };
    }
}

pub async fn new_rendering_engine(
    event_loop: &ActiveEventLoop,
    background_color: Color,
) -> MetallicResult<RenderingEngine> {
    let window = event_loop.create_window(WindowAttributes::default())?;
    let window: &'static _ = Box::leak(Box::new(window));
    let instance = Instance::new(InstanceDescriptor::default());
    let surface = instance.create_surface(window)?;
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .ok_or(MetallicError::NoAdapterFoundError)?;
    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await?;
    let surface_configuration = {
        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .into_iter()
            .find(TextureFormat::is_srgb)
            .ok_or(MetallicError::InvalidConfigurationError(
                InvalidConfigurationError::NoTextureFormatFoundError,
            ))?;
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            desired_maximum_frame_latency: 1,
            view_formats: vec![],
        }
    };
    surface.configure(&device, &surface_configuration);
    let font_system = FontSystem::new();
    let object_engine = ObjectEngine::default();
    Ok(RenderingEngine {
        instance,
        window,
        surface,
        device,
        queue,
        surface_configuration,
        background_color,
        font_system,
        object_engine,
    })
}

pub fn load_font(
    rendering_engine: &mut RenderingEngine,
    path: impl AsRef<Path>,
) -> MetallicResult<()> {
    rendering_engine.font_system.db_mut().load_font_file(path)?;
    Ok(())
}

pub fn resize(rendering_engine: &mut RenderingEngine, new_size: PhysicalSize<u32>) {
    rendering_engine.surface_configuration.width = new_size.width;
    rendering_engine.surface_configuration.height = new_size.height;
    rendering_engine.surface.configure(
        &rendering_engine.device,
        &rendering_engine.surface_configuration,
    );
}

pub fn request_redraw(rendering_engine: &mut RenderingEngine) {
    rendering_engine.window.request_redraw();
}

pub fn render(rendering_engine: &mut RenderingEngine) -> MetallicResult<()> {
    let surface_texture = rendering_engine.surface.get_current_texture()?;
    let view = surface_texture
        .texture
        .create_view(&TextureViewDescriptor::default());
    let mut encoder = rendering_engine
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let _ = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(rendering_engine.background_color),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    };
    let command_buffer = encoder.finish();
    rendering_engine.queue.submit(Some(command_buffer));
    surface_texture.present();
    Ok(())
}

pub fn object_engine(rendering_engine: &mut RenderingEngine) -> &mut ObjectEngine {
    &mut rendering_engine.object_engine
}

// Object Engine
// =============================================================================

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(usize, usize);

#[derive(Default)]
pub struct ObjectEngine {
    object_matrix: BTreeMap<usize, Vec<Object>>,
}

pub fn clear(object_engine: &mut ObjectEngine) {
    object_engine.object_matrix.clear();
}

pub fn add_object(object_engine: &mut ObjectEngine, layer: usize, object: Object) -> Key {
    let object_row = object_engine
        .object_matrix
        .entry(layer)
        .or_insert_with(Vec::default);
    let index = object_row.len();
    object_row.push(object);
    Key(layer, index)
}

pub fn get_object(object_engine: &ObjectEngine, key: Key) -> Option<&Object> {
    let object_row = object_engine.object_matrix.get(&key.0)?;
    let object = object_row.get(key.1)?;
    Some(object)
}

pub fn get_object_mut(object_engine: &mut ObjectEngine, key: Key) -> Option<&mut Object> {
    let object_row = object_engine.object_matrix.get_mut(&key.0)?;
    let object = object_row.get_mut(key.1)?;
    Some(object)
}
