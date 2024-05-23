use metallic::{
    primitives::{point, Properties, Rect, Shape, ShapeType},
    rendering_engine::RenderingEngine,
};
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
            WindowEvent::RedrawRequested => rendering_engine.render()?,
            _ => (),
        };
    };
    Ok(())
}

fn build_initial_scene(rendering_engine: &mut RenderingEngine) {
    rendering_engine.add_shape(Shape {
        properties: Properties { color: Color::RED },
        shape_type: ShapeType::Rect(Rect::with_tl_and_dims(point(0.0, 0.0), 200.0, 200.0)),
    });
    rendering_engine.add_shape(Shape {
        properties: Properties {
            color: Color::WHITE,
        },
        shape_type: ShapeType::Rect(Rect::with_tl_and_dims(point(0.0, 0.0), 100.0, 100.0)),
    });
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
