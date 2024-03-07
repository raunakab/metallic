use wgpu::{Backends, Instance, InstanceDescriptor, PowerPreference, RequestAdapterOptions, DeviceDescriptor, SurfaceConfiguration, TextureUsages};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

pub fn run() -> ! {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    let window: &_ = Box::leak(Box::new(window));

    let size = window.inner_size();

    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(window).unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        power_preference: PowerPreference::LowPower,
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor::default(), None)).unwrap();

    let mut surface_configuration = surface.get_default_config(&adapter, size.width, size.height).unwrap();
    // surface.configure(&device, &surface_configuration);

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run(|event, target| {
            match event {
                Event::LoopExiting
                | Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                }
                | Event::WindowEvent {
                    event: WindowEvent::Destroyed,
                    ..
                } => target.exit(),

                Event::AboutToWait => window.request_redraw(),

                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    // redraw
                }

                Event::WindowEvent { event: WindowEvent::Resized(PhysicalSize { width, height }), .. } => {
                    // surface_configuration.width = width;
                    // surface_configuration.height = height;
                    // surface.configure(&device, &surface_configuration);
                },

                _ => (),
            };
        })
        .unwrap();

    unreachable!()
}
