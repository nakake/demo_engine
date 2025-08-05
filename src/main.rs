use winit::event_loop;

mod app;
mod core;
mod graphics;
mod resources;
mod window;

use core::error::EngineError;

fn main() -> Result<(), EngineError> {
    let event_loop = event_loop::EventLoop::new()
        .map_err(|e| EngineError::EventLoopCreation(format!("Event loop creation error: {}", e)))?;
    let mut app = app::app::App::new();

    event_loop.run_app(&mut app)
        .map_err(|e| EngineError::EventLoopRun(format!("Event loop run error: {}", e)))?;
    
    Ok(())
}
