use std::mem::size_of;

use bytemuck::cast_slice;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, VertexBuffers};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Face, FragmentState, FrontFace,
    IndexFormat, Instance, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, StoreOp, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, VertexBufferLayout,
    VertexState, VertexStepMode,
};
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::Window};

use crate::{
    primitives::{to_vertex, Ctor, Object, Vertex},
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

pub async fn new_wgpu_bundle(event_loop: &ActiveEventLoop) -> MetallicResult<WgpuBundle> {
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
    let shader = device.create_shader_module(include_wgsl!("shaders/main.wgsl"));
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

struct SceneBundle {
    background_color: Color,
    shapes: Vec<(Object, usize)>,
    layer: usize,
    fill_tessellator: FillTessellator,
}

pub struct GlyphBundle;

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
        let wgpu_bundle = new_wgpu_bundle(event_loop).await?;
        Ok(Self {
            wgpu_bundle,
            scene_bundle: SceneBundle {
                background_color,
                shapes: vec![],
                layer: 0,
                fill_tessellator: FillTessellator::default(),
            },
            glyph_bundle: GlyphBundle,
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

    pub fn load_font(&mut self) {}

    pub fn add_object(&mut self, object: Object) {
        let layer = self.scene_bundle.layer;
        let index = match self
            .scene_bundle
            .shapes
            .binary_search_by(|&(_, curr_layer)| curr_layer.cmp(&layer))
        {
            Ok(index) => index + 1,
            Err(index) => index,
        };
        self.scene_bundle.shapes.insert(index, (object, layer));
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
    for (object, _) in &rendering_engine.scene_bundle.shapes {
        match object {
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
