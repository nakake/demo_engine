use winit::event_loop;

mod app;
mod graphics;
mod resources;
mod window;

fn main() {
    let event_loop = event_loop::EventLoop::new().unwrap();
    let mut app = app::app::App::new();

    event_loop.run_app(&mut app).unwrap();
}
