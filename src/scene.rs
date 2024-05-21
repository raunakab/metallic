pub mod colors {
    #![allow(dead_code)]

    use crate::scene::Color;

    pub const BLACK: Color = [0., 0., 0., 1.];
    pub const BLUE: Color = [0., 0., 1., 1.];
    pub const GREEN: Color = [0., 1., 0., 1.];
    pub const CYAN: Color = [0., 1., 1., 1.];
    pub const RED: Color = [1., 0., 0., 1.];
    pub const PURPLE: Color = [1., 0., 1., 1.];
    pub const YELLOW: Color = [1., 1., 0., 1.];
    pub const WHITE: Color = [1., 1., 1., 1.];
}

use std::{collections::VecDeque, mem::size_of, rc::Rc};

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BlendState, Buffer, BufferUsages, Color as WgpuColor, ColorTargetState,
    ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Face, FragmentState,
    FrontFace, Instance, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, StoreOp, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, VertexBufferLayout,
    VertexState, VertexStepMode,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
    event_loop::ActiveEventLoop,
    window::Window,
};

pub type Point = [f32; 2];
pub type Color = [f32; 4];

const IO_EVENTS_CAPACITY: usize = 8;

// pub struct SceneEngine;
// #[derive(Default)]
// pub struct IoEngine {
//     relative_cursor_position: Option<PhysicalPosition<f32>>,
//     io_events: VecDeque<IoEvent>,
// }
// pub fn register_io_event(rendering_engine: &mut RenderingEngine, io_event:
// IoEvent) {     let io_events = &mut rendering_engine.io_engine.io_events;
//     while io_events.len() >= IO_EVENTS_CAPACITY {
//         io_events.pop_front();
//     }
//     io_events.push_back(io_event);
// }
// fn handle_io_events(rendering_engine: &mut RenderingEngine) {
//     while let Some(io_event) =
// rendering_engine.io_engine.io_events.pop_front() {         match io_event {
//             IoEvent::CursorMoved(CursorMoved { position }) => {
//                 let size = rendering_engine.window.inner_size();
//                 let relative_cursor_position = absolute_to_relative(size,
// position);                 
// rendering_engine.io_engine.relative_cursor_position =
// Some(relative_cursor_position);             },
//             IoEvent::MouseInput(MouseInput { state, button }) => {
//                 todo!()
//             },
//         }
//     }
// }
// impl IoEngine {
//     pub fn submit_io_event(&mut self, io_event: IoEvent) {
//         while self.io_events.len() >= IO_EVENTS_CAPACITY {
//             self.io_events.pop_front();
//         }
//         if let IoEvent::CursorMoved(CursorMoved { position }) = io_event {
//         };
//         self.io_events.push_back(io_event);
//     }
// }

pub struct Scene(Vec<Rectangle>);

impl Scene {
    pub fn add_rectangle(&mut self, rectangle: Rectangle) {
        self.0.push(rectangle);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

pub struct Rectangle {
    pub top_left: Point,
    pub bottom_right: Point,
    pub color: Color,
    pub on_touch: Option<Rc<dyn for<'a> Fn(&'a mut Scene)>>,
}

#[repr(C)]
#[derive(Debug, Clone, Pod, Zeroable, Copy, PartialEq)]
pub struct Vertex {
    point: Point,
    color: Color,
}

pub struct RenderingEngine {
    background_color: WgpuColor,
    instance: Instance,
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    surface_configuration: SurfaceConfiguration,
    shader: ShaderModule,
    render_pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
    scene: Scene,
    user_inputs: VecDeque<IoEvent>,
    cursor_position: Option<PhysicalPosition<f64>>,
    // io_engine: IoEngine,
}

impl Drop for RenderingEngine {
    fn drop(&mut self) {
        let window = self.window as *const _ as *mut Window;
        let _ = unsafe { Box::from_raw(window) };
    }
}

impl RenderingEngine {
    pub async fn new(
        event_loop: &ActiveEventLoop,
        background_color: WgpuColor,
    ) -> anyhow::Result<Self> {
        create_rendering_engine(event_loop, background_color).await
    }

    pub fn scene(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn redraw(&self) {
        self.window.request_redraw();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        render(self)
    }

    pub fn submit_user_input(&mut self, user_input: IoEvent) {
        submit_user_input(self, user_input)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum IoEvent {
    CursorMoved(CursorMoved),
    MouseInput(MouseInput),
}

#[derive(Clone, Copy, PartialEq)]
pub struct CursorMoved {
    pub position: PhysicalPosition<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}

fn submit_user_input(rendering_engine: &mut RenderingEngine, user_input: IoEvent) {
    if rendering_engine.user_inputs.len() == IO_EVENTS_CAPACITY {
        rendering_engine.user_inputs.pop_front();
    };
    rendering_engine.user_inputs.push_back(user_input);
}

fn handle_user_inputs(rendering_engine: &mut RenderingEngine) {
    while let Some(user_input) = rendering_engine.user_inputs.pop_front() {
        match user_input {
            IoEvent::CursorMoved(CursorMoved { position }) => {
                rendering_engine.cursor_position = Some(position)
            }
            IoEvent::MouseInput(MouseInput { .. }) => {
                if let Some(absolute_cursor_position) = rendering_engine.cursor_position {
                    run_callback(
                        &mut rendering_engine.scene,
                        rendering_engine.window.inner_size(),
                        absolute_cursor_position,
                    );
                };
            }
        }
    }
}

fn absolute_to_relative(
    size: PhysicalSize<u32>,
    absolute_position: PhysicalPosition<f64>,
) -> PhysicalPosition<f32> {
    fn absolute_to_relative_1d(a: f64, length: u32) -> f32 {
        ((a / (length as f64)) * 2. - 1.) as _
    }

    let x = absolute_to_relative_1d(absolute_position.x, size.width);
    let y = -absolute_to_relative_1d(absolute_position.y, size.height);
    PhysicalPosition { x, y }
}

fn run_callback(
    scene: &mut Scene,
    size: PhysicalSize<u32>,
    absolute_position: PhysicalPosition<f64>,
) {
    fn does_hit(rectangle: &Rectangle, PhysicalPosition { x, y }: PhysicalPosition<f64>) -> bool {
        let x = x as f32;
        let y = y as f32;
        let [x1, y1] = rectangle.top_left;
        let [x2, y2] = rectangle.bottom_right;
        x1 <= x && x <= x2 && y2 <= y && y <= y1
    }

    let relative_position = absolute_to_relative(size, absolute_position);
    // let mut callback = None;
    for rectangle in &scene.0 {
        // if does_hit(rectangle, relative_position) {
        //     if let Some(on_touch) = rectangle.on_touch.as_ref() {
        //         callback = Some(on_touch.clone());
        //     };
        //     break;
        // }
    }
    // if let Some(callback) = callback {
    //     callback(scene);
    // };
}

fn render(rendering_engine: &mut RenderingEngine) -> anyhow::Result<()> {
    handle_user_inputs(rendering_engine);
    let (buffer, number_of_vertices) =
        create_buffer(&rendering_engine.scene, &rendering_engine.device);
    let surface_texture = rendering_engine.surface.get_current_texture()?;
    let texture_view = surface_texture
        .texture
        .create_view(&TextureViewDescriptor::default());
    let mut encoder = rendering_engine
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(rendering_engine.background_color),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        render_pass.set_pipeline(&rendering_engine.render_pipeline);
        render_pass.set_vertex_buffer(0, buffer.slice(..));
        render_pass.draw(0..(number_of_vertices as _), 0..1);
    };
    rendering_engine.queue.submit([encoder.finish()]);
    surface_texture.present();
    Ok(())
}

async fn create_rendering_engine(
    event_loop: &ActiveEventLoop,
    background_color: WgpuColor,
) -> anyhow::Result<RenderingEngine> {
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
                attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x4],
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
    Ok(RenderingEngine {
        background_color,
        instance,
        window,
        surface,
        device,
        queue,
        surface_configuration,
        shader,
        render_pipeline_layout,
        render_pipeline,
        scene: Scene(vec![]),
        user_inputs: VecDeque::with_capacity(IO_EVENTS_CAPACITY),
        cursor_position: None,
        // io_engine: IoEngine::default(),
    })
}

fn create_buffer(scene: &Scene, device: &Device) -> (Buffer, usize) {
    let vertices = scene.0.iter().flat_map(create_vertices).collect::<Vec<_>>();
    let number_of_vertices = vertices.len();
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: &cast_slice::<_, u8>(vertices.as_slice()),
        usage: BufferUsages::VERTEX,
    });
    (buffer, number_of_vertices)
}

fn create_vertices(rectangle: &Rectangle) -> [Vertex; 6] {
    let [x0, y0] = rectangle.top_left;
    let [x1, y1] = rectangle.bottom_right;
    let tl = [x0, y0];
    let tr = [x1, y0];
    let bl = [x0, y1];
    let br = [x1, y1];
    let v1 = Vertex {
        point: tl,
        color: rectangle.color,
    };
    let v2 = Vertex {
        point: br,
        color: rectangle.color,
    };
    let v3 = Vertex {
        point: tr,
        color: rectangle.color,
    };
    let v4 = Vertex {
        point: tl,
        color: rectangle.color,
    };
    let v5 = Vertex {
        point: bl,
        color: rectangle.color,
    };
    let v6 = Vertex {
        point: br,
        color: rectangle.color,
    };
    [v1, v2, v3, v4, v5, v6]
}
