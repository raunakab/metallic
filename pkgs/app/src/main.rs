use euclid::default::Point2D;
use lyon::path::{Path, Winding};
use metallic::{primitives::{Object, Shape}, rendering_engine::RenderingEngine};
use pollster::block_on;
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Err(error) = handle_window_event(self, event_loop, event) {
            panic!("Error handling window event: {:?}", error);
        }
    }
}

async fn resume(app: &mut App, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
    let mut rendering_engine = RenderingEngine::new(event_loop, Color::BLACK).await?;
    init_rendering_engine(&mut rendering_engine)?;
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
            WindowEvent::RedrawRequested => rendering_engine.render()?,
            _ => (),
        };
    };
    Ok(())
}

fn init_rendering_engine(rendering_engine: &mut RenderingEngine) -> anyhow::Result<()> {
    rendering_engine.load_font("assets/Roboto-Regular.ttf")?;
    rendering_engine.add_object({
        let mut builder = Path::builder();
        builder.add_circle(Point2D::new(100.0, 100.0), 100.0, Winding::Positive);
        let path = builder.build();
        Object::Shape(Shape {
            path,
            color: Color::RED,
        })
    });
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
