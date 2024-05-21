use pollster::block_on;
use todo_app::{
    primitives::{Point, PointFormat, Rect},
    rendering_engine::RenderingEngine,
};
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    event_loop::{ControlFlow, EventLoop},
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
        // let mut submit = |user_input| {
        //     // rendering_engine.submit_user_input(user_input);
        //     // rendering_engine.redraw();
        // };

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                app.0 = None;
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                rendering_engine.resize(new_size);
                rendering_engine.redraw();
            },
            // WindowEvent::CursorMoved { position, .. } => {
            //     // submit(IoEvent::CursorMoved(CursorMoved { position }))
            // }
            // WindowEvent::MouseInput { .. } => {
            //     submit(IoEvent::MouseInput(MouseInput { state, button }))
            // }
            WindowEvent::RedrawRequested => rendering_engine.render()?,
            _ => (),
        };
    };
    Ok(())
}

fn build_initial_scene(rendering_engine: &mut RenderingEngine) {
    rendering_engine.add_rect(Rect {
        tl: Point {
            x: 0.0,
            y: 0.0,
            point_format: PointFormat::Absolute,
        },
        br: Point {
            x: 100.0,
            y: 100.0,
            point_format: PointFormat::Absolute,
        },
        color: Color::WHITE,
    });
    rendering_engine.add_rect(Rect {
        tl: Point {
            x: 0.0,
            y: 100.0,
            point_format: PointFormat::Absolute,
        },
        br: Point {
            x: 100.0,
            y: 200.0,
            point_format: PointFormat::Absolute,
        },
        color: Color::RED,
    });
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
