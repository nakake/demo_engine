use std::sync::Arc;

use winit::window::Window as WinitWindow;

#[derive(Clone)]
pub struct Window {
    pub window: Arc<WinitWindow>,
}

impl Window {
    pub fn new(window: Arc<WinitWindow>) -> Self {
        Window { window }
    }

    pub fn get_window(&self) -> Arc<WinitWindow> {
        self.window.clone()
    }
}
