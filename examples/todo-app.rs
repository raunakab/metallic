use lyon::{
    geom::Box2D,
    math::{Point, Size},
    path::{Path, Winding},
};
use metallic::{
    primitives::{shape, text, Brush, Text},
    rendering_engine::{
        new_rendering_engine, object_engine::add_object, render, request_redraw, resize,
        RenderingEngine,
        object_engine,
    },
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
        if let Err(error) = block_on(resume(self, event_loop)) {
            eprintln!("{:?}", error);
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Err(error) = handle_window_event(self, event_loop, event) {
            eprintln!("{:?}", error);
        };
    }
}

async fn resume(app: &mut App, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
    let mut rendering_engine = new_rendering_engine(event_loop, Color::WHITE).await?;
    init(&mut rendering_engine);
    app.rendering_engine = Some(rendering_engine);
    Ok(())
}

fn handle_window_event(
    app: &mut App,
    event_loop: &ActiveEventLoop,
    event: WindowEvent,
) -> anyhow::Result<()> {
    if let Some(rendering_engine) = app.rendering_engine.as_mut() {
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => exit(app, event_loop),
            WindowEvent::Resized(new_size) => {
                resize(rendering_engine, new_size);
                request_redraw(rendering_engine);
            }
            WindowEvent::KeyboardInput { event, .. } => match event.logical_key {
                Key::Named(NamedKey::Control) => app.control_element_state = Some(event.state),
                Key::Character(c)
                    if app.control_element_state == Some(ElementState::Pressed) && c == "w" =>
                {
                    exit(app, event_loop);
                }
                _ => (),
            },
            WindowEvent::RedrawRequested => {
                render(rendering_engine)?;
            }
            _ => (),
        }
    };
    Ok(())
}

fn init(rendering_engine: &mut RenderingEngine) {
    let object_engine = object_engine(rendering_engine);
    let _ = add_object(
        object_engine,
        0,
        shape(
            {
                let mut path = Path::builder();
                path.add_circle(Point::splat(100.0), 10.0, Winding::Positive);
                path.build()
            },
            Brush::Solid(Color::BLUE),
        ),
    );
    let _ = add_object(
        object_engine,
        0,
        shape(
            {
                let mut path = Path::builder();
                path.add_rectangle(
                    &Box2D::from_origin_and_size(Point::zero(), Size::splat(100.0)),
                    Winding::Positive,
                );
                path.build()
            },
            Brush::Solid(Color::GREEN),
        ),
    );
    let _ = add_object(
        object_engine,
        0,
        text(
            Text {
                text: "Hello, Prasad!".into(),
                ..Default::default()
            },
            Brush::Solid(Color::RED),
        ),
    );
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
