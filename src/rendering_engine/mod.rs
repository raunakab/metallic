pub mod object_engine;

use std::{mem::size_of, path::Path};

use bytemuck::{cast_slice, Pod, Zeroable};
use glyphon::FontSystem;
use lyon::{
    algorithms::rounded_polygon::Point,
    tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, FragmentState,
    IndexFormat, Instance, InstanceDescriptor, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PresentMode, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
    TextureViewDescriptor, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    primitives::{Brush, ObjectKind},
    rendering_engine::object_engine::ObjectEngine,
    InvalidConfigurationError, MetallicError, MetallicResult,
};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

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
    let size = rendering_engine.window.inner_size();
    let surface_texture = rendering_engine.surface.get_current_texture()?;
    let view = surface_texture
        .texture
        .create_view(&TextureViewDescriptor::default());
    let mut encoder = rendering_engine
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut buffers: Vec<(Buffer, Buffer, usize)> = vec![];
        let shader = rendering_engine
            .device
            .create_shader_module(include_wgsl!("../shaders/solid.wgsl"));
        for (_, object_row) in &rendering_engine.object_engine.object_matrix {
            for object in object_row {
                let Color { r, g, b, a } = match object.brush {
                    Brush::Solid(color) => color,
                };
                match &object.object_kind {
                    ObjectKind::Shape(path) => {
                        let mut geometry = VertexBuffers::<_, u16>::new();
                        let mut buffers_builder =
                            BuffersBuilder::new(&mut geometry, |fill_vertex: FillVertex| {
                                let Point { x, y, .. } = fill_vertex.position();
                                let convert = |a: f32, length: u32| -> f32 {
                                    (a / (length as f32)) * 2.0 - 1.0
                                };
                                let x = convert(x, size.width);
                                let y = -convert(y, size.height);
                                Vertex {
                                    position: [x, y],
                                    color: [r as _, g as _, b as _, a as _],
                                }
                            });
                        let mut tessellator = FillTessellator::new();
                        tessellator.tessellate_path(
                            path,
                            &FillOptions::default(),
                            &mut buffers_builder,
                        )?;
                        let len = geometry.indices.len();
                        let vertex_buffer =
                            rendering_engine
                                .device
                                .create_buffer_init(&BufferInitDescriptor {
                                    label: None,
                                    contents: cast_slice(&geometry.vertices),
                                    usage: BufferUsages::VERTEX,
                                });
                        let index_buffer =
                            rendering_engine
                                .device
                                .create_buffer_init(&BufferInitDescriptor {
                                    label: None,
                                    contents: cast_slice(&geometry.indices),
                                    usage: BufferUsages::INDEX,
                                });
                        buffers.push((vertex_buffer, index_buffer, len));
                    }
                }
            }
        }
        let render_pipeline_layout = rendering_engine
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor::default());
        let render_pipeline =
            rendering_engine
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&render_pipeline_layout),
                    vertex: VertexState {
                        module: &shader,
                        entry_point: "vs",
                        compilation_options: PipelineCompilationOptions::default(),
                        buffers: &[VertexBufferLayout {
                            array_stride: size_of::<Vertex>() as _,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                        }],
                    },
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        module: &shader,
                        entry_point: "fs",
                        compilation_options: PipelineCompilationOptions::default(),
                        targets: &[Some(ColorTargetState {
                            format: rendering_engine.surface_configuration.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                });
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
        for &(ref vertex_buffer, ref index_buffer, len) in &buffers {
            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..(len as _), 0, 0..1);
        }
    };
    let command_buffer = encoder.finish();
    rendering_engine.queue.submit(Some(command_buffer));
    surface_texture.present();
    Ok(())
}

pub fn object_engine(rendering_engine: &mut RenderingEngine) -> &mut ObjectEngine {
    &mut rendering_engine.object_engine
}
