use std::sync::Arc;

use winit::{application::ApplicationHandler, window::WindowAttributes};

use crate::{
    core::{config::AppConfig, logging::init_logger},
    graphics::engine::GraphicsEngine,
    input::InputState,
    scene::{SceneId, demo_scene::DemoScene, manager::SceneManager},
    window::Window,
};

pub struct App {
    window: Option<Window>,
    engine: Option<GraphicsEngine>,
    input_state: InputState,
    last_frame_time: std::time::Instant,
    scene_manager: SceneManager,
    config: Arc<AppConfig>,
}

impl App {
    pub fn new() -> Self {
        init_logger();

        App {
            window: None,
            engine: None,
            input_state: InputState::new(),
            last_frame_time: std::time::Instant::now(),
            scene_manager: SceneManager::new(),
            config: Arc::new(AppConfig::load_or_default("config.toml")),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let winit_window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(self.config.window.title.clone())
                        .with_inner_size(winit::dpi::PhysicalSize::new(
                            self.config.window.width,
                            self.config.window.height,
                        ))
                        .with_resizable(self.config.window.resizable),
                )
                .map_err(|e| {
                    log::error!("Window creation error: {}", e);
                })
                .unwrap(),
        );

        let scene_id = SceneId::new("Demo_Scene");
        let demo_scene = Box::new(DemoScene::new(
            self.config.window.width as f32 / self.config.window.height as f32,
            self.config.clone(),
        ));

        self.scene_manager.register_scene(scene_id, demo_scene);
        if let Err(e) = self.scene_manager.set_current_scene(scene_id) {
            log::error!("Failed to set current scene: {}", e);
            return;
        }

        let window = Window::new(winit_window);

        let current_scene = self
            .scene_manager
            .take_current_scene()
            .expect("No current scene set");

        let engine = match pollster::block_on(GraphicsEngine::new(
            window.clone(),
            current_scene,
            &self.config.rendering,
        )) {
            Ok(engine) => engine,
            Err(e) => {
                log::error!("Graphics engine initialization error: {}", e);
                return;
            }
        };

        self.window = Some(window.clone());
        self.engine = Some(engine);

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
                        log::error!("Rendering error: {}", e);
                    }
                }

                self.input_state.reset_mouse_delta();

                // 継続的なレンダリングのため次フレームをリクエスト
                if let Some(window) = &self.window {
                    window.get_window().request_redraw();
                }
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                log::debug!("KeyboardInput event received: {:?}", event);
                self.input_state.process_keybord(&event);

                if self
                    .input_state
                    .is_key_pressed(winit::keyboard::KeyCode::Escape)
                {
                    event_loop.exit();
                }

                // キー入力後に再描画をリクエスト
                if let Some(window) = &self.window {
                    window.get_window().request_redraw();
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                self.input_state.process_mouse_input(button, state);
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.input_state
                    .set_mouse_position(position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }
}
