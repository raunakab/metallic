use metallic::create_engine;
use pollster::block_on;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::Window,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();

    let engine = block_on(create_engine(&window)).unwrap();
    let mut i = 0;

    event_loop
        .run(move |event, target| {
            match event {
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
                _ => {
                    println!("Looping {}", i);
                    target.set_control_flow(ControlFlow::Poll);
                    i += 1;
                },
            }
        })
        .unwrap();
}
