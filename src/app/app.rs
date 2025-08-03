use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    window::{self, WindowAttributes},
};

use crate::{graphics::engine::GraficsEngine, window::window::Window};

pub struct App {
    window: Option<Window>,
    engine: Option<GraficsEngine>,
}

impl App {
    pub fn new() -> Self {
        App {
            window: None,
            engine: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let winit_window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Demo Engine")
                        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
                )
                .unwrap(),
        );

        let window = Window::new(winit_window);
        let engine = pollster::block_on(GraficsEngine::new(window.clone()));

        self.window = Some(window);
        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                if let Some(engine) = &mut self.engine {
                    engine.resize(size.width, size.height);
                }
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(engine) = &mut self.engine {
                    engine.render();
                }
            }
            _ => {}
        }
    }
}
