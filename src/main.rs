use std::rc::Rc;

use pollster::block_on;
use todo_app::{
    primitives::{AbsPoint, IoEvent, MouseInput, Properties, Rect, Shape, ShapeType},
    rendering_engine::RenderingEngine,
};
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

#[derive(Default)]
pub struct App(Option<RenderingEngine>);

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = block_on(resume(self, event_loop)) {
            panic!("Error resuming application: {:?}", error);
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, _: ()) {
        if let Some(rendering_engine) = self.0.as_mut() {
            rendering_engine.redraw();
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Err(error) = handle_window_event(self, event_loop, event) {
            panic!("Error handling window event: {:?}", error);
        }
    }
}

async fn resume(app: &mut App, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
    let mut rendering_engine = RenderingEngine::new(event_loop, Color::BLACK).await?;
    build_initial_scene(&mut rendering_engine);
    app.0 = Some(rendering_engine);
    Ok(())
}

fn handle_window_event(
    app: &mut App,
    event_loop: &ActiveEventLoop,
    event: WindowEvent,
) -> anyhow::Result<()> {
    if let Some(rendering_engine) = app.0.as_mut() {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                app.0 = None;
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                rendering_engine.resize(new_size);
                rendering_engine.redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let io_event = IoEvent::CursorMoved(position.into());
                rendering_engine.register_io_event(io_event);
                rendering_engine.redraw();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let io_event = IoEvent::MouseInput(MouseInput { state, button });
                rendering_engine.register_io_event(io_event);
                rendering_engine.redraw();
            }
            WindowEvent::RedrawRequested => rendering_engine.render()?,
            _ => (),
        };
    };
    Ok(())
}

fn build_initial_scene(rendering_engine: &mut RenderingEngine) {
    rendering_engine.add_shape(Shape {
        properties: Properties {
            color: Color::WHITE,
            on_mouse_input: Some(Rc::new(|mouse_input| match mouse_input.state {
                ElementState::Pressed => println!("White pressed"),
                ElementState::Released => println!("White released"),
            })),
        },
        shape_type: ShapeType::Rect(Rect {
            tl: AbsPoint(PhysicalPosition { x: 0.0, y: 0.0 }),
            br: AbsPoint(PhysicalPosition { x: 100.0, y: 100.0 }),
        }),
    });
    rendering_engine.add_shape(Shape {
        properties: Properties {
            color: Color::RED,
            on_mouse_input: Some(Rc::new(|mouse_input| match mouse_input.state {
                ElementState::Pressed => println!("Red pressed"),
                ElementState::Released => println!("Red released"),
            })),
        },
        shape_type: ShapeType::Rect(Rect {
            tl: AbsPoint(PhysicalPosition { x: 0.0, y: 100.0 }),
            br: AbsPoint(PhysicalPosition { x: 100.0, y: 200.0 }),
        }),
    });
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
