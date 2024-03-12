use metallic::Engine;
use pollster::block_on;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();

    let mut engine = Engine::new(&window).await.unwrap();

    event_loop
        .run(move |event, target| match event {
            Event::LoopExiting
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            } => {
                target.exit();
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                engine.render().unwrap();
                target.set_control_flow(ControlFlow::Poll);
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                engine.resize(size);
                target.set_control_flow(ControlFlow::Poll);
            },
            _ => target.set_control_flow(ControlFlow::Wait),
        })
        .unwrap();
}

fn main() {
    block_on(run());
}
