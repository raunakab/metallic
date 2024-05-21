use std::mem::size_of;

use wgpu::{
    include_wgsl, BlendState, ColorTargetState, ColorWrites, Device, DeviceDescriptor, Face,
    FragmentState, FrontFace, Instance, MultisampleState, PipelineCompilationOptions,
    PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModule, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, VertexBufferLayout,
    VertexState, VertexStepMode,
};
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::rendering_engine::Vertex;

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

pub async fn new_wgpu_bundle(event_loop: &ActiveEventLoop) -> anyhow::Result<WgpuBundle> {
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
        .ok_or_else(im_lazy!())?;
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
            .ok_or_else(im_lazy!())?;
        let present_mode = capabilities
            .present_modes
            .into_iter()
            .find(|&present_mode| present_mode == PresentMode::Fifo)
            .ok_or_else(im_lazy!())?;
        let &alpha_mode = capabilities.alpha_modes.first().ok_or_else(im_lazy!())?;
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
