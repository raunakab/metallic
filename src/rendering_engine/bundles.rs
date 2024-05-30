use std::mem::size_of;

use bytemuck::cast_slice;
use cosmic_text::{Attrs, Buffer as CTBuffer, Color as CTColor, FontSystem, LayoutGlyph, Metrics, Shaping, SwashCache};
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, VertexBuffers};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites, Device,
    DeviceDescriptor, Face, FragmentState, FrontFace, Instance, MultisampleState,
    PipelineCompilationOptions, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PresentMode,
    PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModule, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::{
    primitives::{to_vertex, Ctor, LayeredObject, Object, Text, Vertex, color_converter},
    rendering_engine::RenderingEngine,
    InvalidConfigurationError, MetallicError, MetallicResult,
};

pub struct WgpuBundle {
    pub instance: Instance,
    pub window: &'static Window,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub surface_configuration: SurfaceConfiguration,
    pub shader: ShaderModule,
    pub render_pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
}

impl Drop for WgpuBundle {
    fn drop(&mut self) {
        let window = self.window as *const _ as *mut Window;
        let _ = unsafe { Box::from_raw(window) };
    }
}

pub struct SceneBundle {
    pub background_color: Color,
    pub layered_objects: Vec<LayeredObject>,
    pub layer: usize,
    pub fill_tessellator: FillTessellator,
}

pub struct GlyphBundle {
    pub font_system: FontSystem,
}

pub struct BufferBundle {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_buffer_size: usize,
}

pub fn create_glyph_bundle() -> GlyphBundle {
    GlyphBundle {
        font_system: FontSystem::new(),
    }
}

pub fn create_scene_bundle(background_color: Color) -> SceneBundle {
    SceneBundle {
        background_color,
        layered_objects: vec![],
        layer: 0,
        fill_tessellator: FillTessellator::default(),
    }
}

pub fn create_buffer_bundle(
    rendering_engine: &mut RenderingEngine,
) -> MetallicResult<BufferBundle> {
    let size = rendering_engine.wgpu_bundle.window.inner_size();
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut offset = 0;
    for layered_object in &rendering_engine.scene_bundle.layered_objects {
        match &layered_object.object {
            Object::Shape(shape) => {
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
            &Object::Text(Text {
                ref text,
                font_size,
                line_height,
                color,
            }) => {
                let mut swash_cache = SwashCache::new();
                let metrics = Metrics {
                    font_size,
                    line_height,
                };
                let mut buffer = CTBuffer::new(&mut rendering_engine.glyph_bundle.font_system, metrics);
                let mut buffer = buffer.borrow_with(&mut rendering_engine.glyph_bundle.font_system);
                buffer.set_size(size.width as _, size.height as _);
                let attrs = Attrs::new();
                buffer.set_text(&text, attrs, Shaping::Advanced);
                buffer.shape_until_scroll(true);
                // buffer.glyphs();
                for run in buffer.layout_runs() {
                    for glyph in run.glyphs {
                        // glyph.
                        // LayoutGlyph;
                    }
                }
                // buffer.draw(&mut swash_cache, color_converter(color), |x, y, w, h, color| {});

                todo!()
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
                contents: &cast_slice(&indices),
                usage: BufferUsages::INDEX,
            });
    Ok(BufferBundle {
        vertex_buffer,
        index_buffer,
        index_buffer_size: indices.len(),
    })
}

pub async fn create_wgpu_bundle(event_loop: &ActiveEventLoop) -> MetallicResult<WgpuBundle> {
    let instance = Instance::default();
    let window = event_loop.create_window(Window::default_attributes())?;
    let window: &'static _ = Box::leak(Box::new(window));
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
    let surface_configuration =
        {
            let size = window.inner_size();
            let capabilities = surface.get_capabilities(&adapter);
            let format = capabilities
                .formats
                .into_iter()
                .find(TextureFormat::is_srgb)
                .ok_or(MetallicError::InvalidConfigurationError(
                    InvalidConfigurationError::NoTextureFormatFoundError,
                ))?;
            let present_mode = capabilities
                .present_modes
                .into_iter()
                .find(|&present_mode| present_mode == PresentMode::Fifo)
                .ok_or(MetallicError::InvalidConfigurationError(
                    InvalidConfigurationError::NoFifoPresentModeFoundError,
                ))?;
            let &alpha_mode = capabilities.alpha_modes.first().ok_or(
                MetallicError::InvalidConfigurationError(
                    InvalidConfigurationError::NoAlphaModeFoundError,
                ),
            )?;
            SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                width: size.width,
                height: size.height,
                present_mode,
                alpha_mode,
                desired_maximum_frame_latency: 1,
                view_formats: vec![],
            }
        };
    surface.configure(&device, &surface_configuration);
    let shader = device.create_shader_module(include_wgsl!("../shaders/main.wgsl"));
    let render_pipeline_layout =
        device.create_pipeline_layout(&PipelineLayoutDescriptor::default());
    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&render_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: "vs",
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[VertexBufferLayout {
                array_stride: size_of::<Vertex>() as _,
                step_mode: VertexStepMode::Vertex,
                attributes: &Vertex::VERTEX_ATTRS,
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fs",
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: surface_configuration.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });
    Ok(WgpuBundle {
        instance,
        window,
        surface,
        device,
        queue,
        surface_configuration,
        shader,
        render_pipeline_layout,
        render_pipeline,
    })
}
