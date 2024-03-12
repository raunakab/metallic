use thiserror::Error;
use wgpu::{
    Adapter, Backends, Color, CommandEncoderDescriptor, CreateSurfaceError, Device,
    DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, RequestDeviceError,
    StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, RenderPipeline, RenderPipelineDescriptor, PipelineLayoutDescriptor, VertexState, include_wgsl, PrimitiveState, PrimitiveTopology, FrontFace, Face, PolygonMode, MultisampleState, FragmentState, ColorTargetState, BlendState, ColorWrites,
};
use winit::{window::Window, dpi::PhysicalSize};

pub type MetallicResult<T> = Result<T, MetallicError>;

#[derive(Error, Debug)]
pub enum MetallicError {
    #[error(transparent)]
    CreateSurfaceError(#[from] CreateSurfaceError),

    #[error("No valid adapter found")]
    RequestAdapterError,

    #[error("Surface is not supported by this adapter")]
    SurfaceNotSupportedError,

    #[error(transparent)]
    RequestDeviceError(#[from] RequestDeviceError),

    #[error(transparent)]
    SurfaceError(#[from] SurfaceError),

    #[error("No SRGB formats were found for this surface")]
    NoSrgbFormatsError,

    #[error("FIFO present modes not supported")]
    FifoPresentModeNotSupportedError,

    #[error("No composite alpha modes present")]
    NoCompositeAlphaModesError,
}

#[allow(unused)]
pub struct Engine<'a> {
    window: &'a Window,
    instance: Instance,
    adapter: Adapter,
    surface_configuration: SurfaceConfiguration,
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl<'a> Engine<'a> {
    pub async fn new(window: &'a Window) -> MetallicResult<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .ok_or_else(|| MetallicError::RequestAdapterError)?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await?;

        let size = window.inner_size();

        let (format, present_mode, alpha_mode) = {
            let capabilities = surface.get_capabilities(&adapter);
            let &format = capabilities
                .formats
                .iter()
                .find(|format| format.is_srgb())
                .ok_or_else(|| MetallicError::NoSrgbFormatsError)?;
            let &present_mode = capabilities
                .present_modes
                .iter()
                .find(|&&present_mode| present_mode == PresentMode::Fifo)
                .ok_or_else(|| MetallicError::FifoPresentModeNotSupportedError)?;
            let &alpha_mode = capabilities
                .alpha_modes
                .first()
                .ok_or_else(|| MetallicError::NoCompositeAlphaModesError)?;
            (format, present_mode, alpha_mode)
        };

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            present_mode,
            alpha_mode,
            height: size.height,
            width: size.width,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        surface.configure(&device, &surface_configuration);

        let shader = device.create_shader_module(include_wgsl! { "shader.wgsl" });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor::default());

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[],
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
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            window,
            instance,
            adapter,
            surface_configuration,
            surface,
            device,
            queue,
            render_pipeline,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface.configure(&self.device, &self.surface_configuration);
    }

    pub fn render(&self) -> MetallicResult<()> {
        let output_surface_texture = self.surface.get_current_texture()?;
        let texture_view = output_surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        };

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        output_surface_texture.present();

        Ok(())
    }
}
