use std::collections::HashSet;

use winit::event::ElementState;

use crate::{
    input::{Key, MouseButton},
    math::Vec2

};

pub struct Input {
    down: HashSet<Key>,
    pressed: HashSet<Key>,
    released: HashSet<Key>,

    mouse_position: Option<Vec2>,

    mouse_down: HashSet<MouseButton>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            down: HashSet::new(),
            pressed: HashSet::new(),
            released: HashSet::new(),

            mouse_position: None,

            mouse_down: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_released: HashSet::new(),
        }
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.down.contains(&key)
    }
    
    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }
    
    pub fn key_released(&self, key: Key) -> bool {
        self.released.contains(&key)
    }

    pub fn mouse_position(&self) -> Option<Vec2> {
        self.mouse_position
    }

    pub fn mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_down.contains(&button)
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_released.contains(&button)
    }

    pub(crate) fn key_event(
        &mut self,
        key: Key,
        state: ElementState,
    ) {
        match state {
            ElementState::Pressed => {
                if self.down.insert(key) {
                    self.pressed.insert(key);
                }
            }
        
            ElementState::Released => {
                self.down.remove(&key);
                self.released.insert(key);
            }
        }
    }

    pub(crate) fn mouse_button_event(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) {
        match state {
            ElementState::Pressed => {
                if self.mouse_down.insert(button) {
                    self.mouse_pressed.insert(button);
                }
            }

            ElementState::Released => {
                self.mouse_down.remove(&button);
                self.mouse_released.insert(button);
            }
        }
    }

    pub(crate) fn set_mouse_position(
        &mut self,
        position: Option<Vec2>,
    ) {
        self.mouse_position = position;
    }

    pub(crate) fn end_frame(&mut self) {
        self.pressed.clear();
        self.released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
    }
}