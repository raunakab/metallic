use std::{mem::size_of, path::Path};

use bytemuck::cast_slice;
use glyphon::{
    Attrs, Buffer as GBuffer, Cache, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport
};
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, VertexBuffers};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Face, FragmentState, FrontFace,
    IndexFormat, Instance, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, StoreOp, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use crate::{
    primitives::{convert_color, to_vertex, Ctor, Object, Text, Vertex},
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
    pub object_layers: Vec<Vec<Object>>,
    pub fill_tessellator: FillTessellator,
    pub font_system: FontSystem,
}

pub fn load_font(
    rendering_engine: &mut RenderingEngine,
    path: impl AsRef<Path>,
) -> MetallicResult<()> {
    rendering_engine
        .scene_bundle
        .font_system
        .db_mut()
        .load_font_file(path)?;
    Ok(())
}

pub fn redraw(rendering_engine: &RenderingEngine) {
    rendering_engine.wgpu_bundle.window.request_redraw();
}

pub fn resize(rendering_engine: &mut RenderingEngine, new_size: PhysicalSize<u32>) {
    rendering_engine.wgpu_bundle.surface_configuration.width = new_size.width;
    rendering_engine.wgpu_bundle.surface_configuration.height = new_size.height;
    rendering_engine.wgpu_bundle.surface.configure(
        &rendering_engine.wgpu_bundle.device,
        &rendering_engine.wgpu_bundle.surface_configuration,
    );
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
    let shader = device.create_shader_module(include_wgsl!("../shaders/solid.wgsl"));
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

pub fn create_scene_bundle(background_color: Color) -> SceneBundle {
    SceneBundle {
        background_color,
        object_layers: vec![],
        fill_tessellator: FillTessellator::default(),
        font_system: FontSystem::new(),
    }
}

pub fn render(rendering_engine: &mut RenderingEngine) -> MetallicResult<()> {
    let surface_texture = rendering_engine.wgpu_bundle.surface.get_current_texture()?;
    let view = surface_texture
        .texture
        .create_view(&TextureViewDescriptor::default());
    let mut encoder = rendering_engine
        .wgpu_bundle
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    let first_and_rest = match &*rendering_engine.scene_bundle.object_layers {
        [] => None,
        [first] => Some((first, None)),
        [first, rest @ ..] => Some((first, Some(rest))),
    };
    match first_and_rest {
        Some((first, rest)) => {
            render_layer(RenderLayerArgs {
                window: rendering_engine.wgpu_bundle.window,
                objects: first,
                fill_tessellator: &mut rendering_engine.scene_bundle.fill_tessellator,
                device: &rendering_engine.wgpu_bundle.device,
                queue: &rendering_engine.wgpu_bundle.queue,
                texture_format: rendering_engine.wgpu_bundle.surface_configuration.format,
                encoder: &mut encoder,
                view: &view,
                background_color: Some(rendering_engine.scene_bundle.background_color),
                render_pipeline: &rendering_engine.wgpu_bundle.render_pipeline,
                font_system: &mut rendering_engine.scene_bundle.font_system,
            })?;
            if let Some(rest) = rest {
                for objects in rest {
                    render_layer(RenderLayerArgs {
                        window: rendering_engine.wgpu_bundle.window,
                        objects,
                        fill_tessellator: &mut rendering_engine.scene_bundle.fill_tessellator,
                        device: &rendering_engine.wgpu_bundle.device,
                        queue: &rendering_engine.wgpu_bundle.queue,
                        texture_format: rendering_engine.wgpu_bundle.surface_configuration.format,
                        encoder: &mut encoder,
                        view: &view,
                        background_color: None,
                        render_pipeline: &rendering_engine.wgpu_bundle.render_pipeline,
                        font_system: &mut rendering_engine.scene_bundle.font_system,
                    })?;
                }
            };
        }
        None => {
            render_layer(RenderLayerArgs {
                window: rendering_engine.wgpu_bundle.window,
                objects: &vec![],
                fill_tessellator: &mut rendering_engine.scene_bundle.fill_tessellator,
                device: &rendering_engine.wgpu_bundle.device,
                queue: &rendering_engine.wgpu_bundle.queue,
                texture_format: rendering_engine.wgpu_bundle.surface_configuration.format,
                encoder: &mut encoder,
                view: &view,
                background_color: Some(rendering_engine.scene_bundle.background_color),
                render_pipeline: &rendering_engine.wgpu_bundle.render_pipeline,
                font_system: &mut rendering_engine.scene_bundle.font_system,
            })?;
        }
    };
    let command_buffer = encoder.finish();
    rendering_engine
        .wgpu_bundle
        .queue
        .submit(Some(command_buffer));
    surface_texture.present();
    Ok(())
}

struct RenderLayerArgs<'a> {
    window: &'static Window,
    objects: &'a Vec<Object>,
    fill_tessellator: &'a mut FillTessellator,
    device: &'a Device,
    queue: &'a Queue,
    texture_format: TextureFormat,
    encoder: &'a mut CommandEncoder,
    view: &'a TextureView,
    background_color: Option<Color>,
    render_pipeline: &'a RenderPipeline,
    font_system: &'a mut FontSystem,
}

fn render_layer(
    RenderLayerArgs {
        window,
        objects,
        fill_tessellator,
        device,
        queue,
        texture_format,
        encoder,
        view,
        background_color,
        render_pipeline,
        font_system,
    }: RenderLayerArgs,
) -> MetallicResult<()> {
    let size = window.inner_size();
    let create_buffers_output = create_buffers(CreateBuffersArgs {
        window,
        objects,
        fill_tessellator,
        device,
        font_system,
    })?;
    let mut swash_cache = SwashCache::new();
    let cache = Cache::new(device);
    let mut viewport = Viewport::new(device, &cache);
    viewport.update(queue, Resolution {
        width: size.width,
        height: size.height,
    });
    let mut atlas = TextAtlas::new(device, queue, &cache, texture_format);
    let mut text_renderer =
        TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
    let text_areas = create_buffers_output
        .text_buffers
        .iter()
        .map(|&(ref buffer, color)| TextArea {
            buffer,
            top: 0.0,
            left: 0.0,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: size.width as _,
                bottom: size.height as _,
            },
            default_color: convert_color(color),
        })
        .collect::<Vec<_>>();
    text_renderer.prepare(
        device,
        queue,
        font_system,
        &mut atlas,
        &viewport,
        text_areas,
        &mut swash_cache,
    )?;
    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        color_attachments: &[Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: background_color.map_or(LoadOp::Load, LoadOp::Clear),
                store: StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    render_pass.set_pipeline(render_pipeline);
    render_pass.set_vertex_buffer(0, create_buffers_output.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
        create_buffers_output.index_buffer.slice(..),
        IndexFormat::Uint16,
    );
    render_pass.draw_indexed(0..(create_buffers_output.len as _), 0, 0..1);
    text_renderer.render(&atlas, &viewport, &mut render_pass)?;
    Ok(())
}

struct CreateBuffersArgs<'a> {
    window: &'static Window,
    objects: &'a Vec<Object>,
    fill_tessellator: &'a mut FillTessellator,
    device: &'a Device,
    font_system: &'a mut FontSystem,
}

struct CreateBuffersOutput {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    len: usize,
    text_buffers: Vec<(GBuffer, Color)>,
}

fn create_buffers(
    CreateBuffersArgs {
        window,
        objects,
        fill_tessellator,
        device,
        font_system,
    }: CreateBuffersArgs,
) -> MetallicResult<CreateBuffersOutput> {
    let size = window.inner_size();
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut text_buffers = vec![];
    let mut offset = 0;
    for object in objects {
        match object {
            Object::Shape(shape) => {
                let mut geometry = VertexBuffers::<_, u16>::new();
                let mut buffers_builder = BuffersBuilder::new(&mut geometry, Ctor);
                fill_tessellator.tessellate_path(
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
                let mut text_buffer = GBuffer::new(
                    font_system,
                    Metrics {
                        font_size,
                        line_height,
                    },
                );
                {
                    let mut text_buffer = text_buffer.borrow_with(font_system);
                    text_buffer.set_size(size.width as _, size.height as _);
                    text_buffer.set_text(&text, Attrs::new(), Shaping::Advanced);
                    text_buffer.shape_until_scroll(false);
                };
                text_buffers.push((text_buffer, color));
            }
        }
    }
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: &cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: &cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });
    Ok(CreateBuffersOutput {
        vertex_buffer,
        index_buffer,
        len: indices.len(),
        text_buffers,
    })
}
