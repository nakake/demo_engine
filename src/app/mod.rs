use std::sync::Arc;

use winit::{application::ApplicationHandler, keyboard::KeyCode, window::WindowAttributes};

use crate::{graphics::engine::GraficsEngine, input::InputState, window::Window};

pub struct App {
    window: Option<Window>,
    engine: Option<GraficsEngine>,
    input_state: InputState,
}

impl App {
    pub fn new() -> Self {
        App {
            window: None,
            engine: None,
            input_state: InputState::new(),
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
                        .with_inner_size(winit::dpi::PhysicalSize::new(800.0, 600.0)),
                )
                .map_err(|e| {
                    eprintln!("Window creation error: {}", e);
                })
                .unwrap(),
        );

        let window = Window::new(winit_window);
        let mut engine = match pollster::block_on(GraficsEngine::new(window.clone())) {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("Graphics engine initialization error: {}", e);
                return;
            }
        };

        engine.initial_default_scene();

        self.window = Some(window);
        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
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
                    let mut movement = glam::Vec3::ZERO;
                    if self.input_state.is_key_pressed(KeyCode::KeyW) {
                        movement.z -= 1.0;
                    }
                    if self.input_state.is_key_pressed(KeyCode::KeyS) {
                        movement.z += 1.0;
                    }
                    if self.input_state.is_key_pressed(KeyCode::KeyA) {
                        movement.x -= 1.0;
                    }
                    if self.input_state.is_key_pressed(KeyCode::KeyD) {
                        movement.x += 1.0;
                    }

                    if let Err(e) = engine.render() {
                        eprintln!("Rendering error: {}", e);
                    }
                }

                self.input_state.reset_mouse_delta();
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                self.input_state.process_keybord(&event);

                if self.input_state.is_key_pressed(KeyCode::Escape) {
                    event_loop.exit();
                }
            }
            winit::event::WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                self.input_state.process_mouse_input(button, state);
            }
            winit::event::WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                self.input_state
                    .set_mouse_position(position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }
}
