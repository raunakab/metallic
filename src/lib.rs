use thiserror::Error;
use wgpu::{
    Adapter, Backends, Color, CommandEncoderDescriptor, CreateSurfaceError, Device,
    DeviceDescriptor, Instance, InstanceDescriptor, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, RequestDeviceError,
    StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor,
};
use winit::window::Window;

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

        Ok(Self {
            window,
            instance,
            adapter,
            surface_configuration,
            surface,
            device,
            queue,
        })
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
            #[allow(unused)]
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
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
        };

        let command_buffer = encoder.finish();
        self.queue.submit([command_buffer]);
        output_surface_texture.present();

        Ok(())
    }
}
