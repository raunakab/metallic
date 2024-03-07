use thiserror::Error;
use wgpu::{
    Adapter, Backends, CreateSurfaceError, Device, DeviceDescriptor, Instance, InstanceDescriptor,
    PowerPreference, Queue, RequestAdapterOptions, RequestDeviceError, Surface,
    SurfaceConfiguration,
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
    SurfaceError,

    #[error(transparent)]
    RequestDeviceError(#[from] RequestDeviceError),
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

pub async fn create_engine(window: &Window) -> MetallicResult<Engine> {
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(window)?;

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: PowerPreference::LowPower,
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| MetallicError::RequestAdapterError)?;

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default(), None)
        .await?;

    let size = window.inner_size();

    let surface_configuration = surface
        .get_default_config(&adapter, size.width, size.height)
        .ok_or_else(|| MetallicError::SurfaceError)?;

    Ok(Engine {
        window,
        instance,
        adapter,
        surface_configuration,
        surface,
        device,
        queue,
    })
}
