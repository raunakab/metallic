use metallic::rendering_engine::{
    new_rendering_engine, render, request_redraw, resize, RenderingEngine,
};
use pollster::block_on;
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowId,
};

#[derive(Default)]
pub struct App {
    rendering_engine: Option<RenderingEngine>,
    control_element_state: Option<ElementState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let rendering_engine = block_on(new_rendering_engine(event_loop, Color::WHITE)).unwrap();
        self.rendering_engine = Some(rendering_engine);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(rendering_engine) = self.rendering_engine.as_mut() {
            match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => exit(self, event_loop),
                WindowEvent::Resized(new_size) => {
                    resize(rendering_engine, new_size);
                    request_redraw(rendering_engine);
                }
                WindowEvent::KeyboardInput { event, .. } => match event.logical_key {
                    Key::Named(NamedKey::Control) => self.control_element_state = Some(event.state),
                    Key::Character(c)
                        if self.control_element_state == Some(ElementState::Pressed)
                            && c == "w" =>
                    {
                        exit(self, event_loop);
                    }
                    _ => (),
                },
                WindowEvent::RedrawRequested => {
                    render(rendering_engine).unwrap();
                }
                _ => (),
            }
        };
    }
}

fn exit(app: &mut App, event_loop: &ActiveEventLoop) {
    app.rendering_engine = None;
    event_loop.exit();
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
