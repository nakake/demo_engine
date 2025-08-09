use std::collections::HashSet;

use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputState {
    keys_pressed: HashSet<KeyCode>,
    mouse_buttons: HashSet<MouseButton>,
    mouse_posittion: glam::Vec2,
    mouse_delta: glam::Vec2,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_posittion: glam::Vec2::ZERO,
            mouse_delta: glam::Vec2::ZERO,
        }
    }

    pub fn process_keybord(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    println!("Key pressed: {:?}", keycode);
                    self.keys_pressed.insert(keycode);
                }
                ElementState::Released => {
                    println!("Key released: {:?}", keycode);
                    self.keys_pressed.remove(&keycode);
                }
            }
            println!("Currently pressed keys: {:?}", self.keys_pressed);
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn process_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons.insert(button);
            }
            ElementState::Released => {
                self.mouse_buttons.remove(&button);
            }
        }
    }

    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        let new_position = glam::Vec2::new(x, y);
        self.mouse_delta = new_position - self.mouse_posittion;
        self.mouse_posittion = new_position;
    }

    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = glam::Vec2::ZERO;
    }
}
