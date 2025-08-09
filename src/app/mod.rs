use std::sync::Arc;

use winit::{application::ApplicationHandler, window::WindowAttributes};

use crate::{
    graphics::engine::GraphicsEngine, input::InputState, scene::demo_scene::DemoScene,
    window::Window,
};

pub struct App {
    window: Option<Window>,
    engine: Option<GraphicsEngine>,
    input_state: InputState,
    last_frame_time: std::time::Instant,
}

impl App {
    pub fn new() -> Self {
        App {
            window: None,
            engine: None,
            input_state: InputState::new(),
            last_frame_time: std::time::Instant::now(),
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

        let demo_scene = Box::new(DemoScene::new());

        let window = Window::new(winit_window);
        let engine = match pollster::block_on(GraphicsEngine::new(window.clone(), demo_scene)) {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("Graphics engine initialization error: {}", e);
                return;
            }
        };

        self.window = Some(window.clone());
        self.engine = Some(engine);

        // 初期の再描画をリクエストして継続的なレンダリングを開始
        window.get_window().request_redraw();
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
                    // 実際のdelta timeを計算
                    let now = std::time::Instant::now();
                    let dt = (now - self.last_frame_time).as_secs_f32();
                    self.last_frame_time = now;

                    if let Err(e) = engine.render(dt, &self.input_state) {
                        eprintln!("Rendering error: {}", e);
                    }
                }

                self.input_state.reset_mouse_delta();

                // 次のフレームをリクエストして継続的なレンダリングを維持
                if let Some(window) = &self.window {
                    window.get_window().request_redraw();
                }
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                println!("KeyboardInput event received: {:?}", event);
                self.input_state.process_keybord(&event);

                if self.input_state.is_key_pressed(winit::keyboard::KeyCode::Escape) {
                    event_loop.exit();
                }

                // キー入力後に再描画をリクエスト
                if let Some(window) = &self.window {
                    window.get_window().request_redraw();
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
