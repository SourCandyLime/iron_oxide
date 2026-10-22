use crate::{
    canvas::Canvas,
    gui::{
        Widget, Button
    },
    input::{
        Input,
    },
};

pub struct GUI {
    pub widgets: Vec<Box<dyn Widget>>
}

impl GUI {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new()
        }
    }

    pub fn new_button(&mut self, x: i32, y: i32, width: u32, height: u32, text: String, id: String) {
        self.widgets.push(Box::new(Button::new(x, y, width, height, text, id)));
    }

    pub fn pressed(&mut self, id: &str) -> bool {
        self.widgets
            .iter()
            .find(|widget| widget.id() == id)
            .map(|widget| widget.pressed())
            .unwrap_or(false)
    }

    pub fn down(&mut self, id: &str) -> bool {
        self.widgets
            .iter()
            .find(|widget| widget.id() == id)
            .map(|widget| widget.down())
            .unwrap_or(false)
    }

    pub fn released(&mut self, id: &str) -> bool {
        self.widgets
            .iter()
            .find(|widget| widget.id() == id)
            .map(|widget| widget.released())
            .unwrap_or(false)
    }

    pub fn clicked(&mut self, id: &str) -> bool {
        self.widgets
            .iter()
            .find(|widget| widget.id() == id)
            .map(|widget| widget.clicked())
            .unwrap_or(false)
    }

    pub fn interactible(&mut self, id: &str) -> bool {
        self.widgets
            .iter()
            .find(|widget| widget.id() == id)
            .map(|widget| widget.interactable())
            .unwrap_or(false)
    }
    
    pub fn update(&mut self, input: &Input) {
        for element in &mut self.widgets {
            element.update(input)
        }
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        for element in &self.widgets {
            element.draw(canvas);
        }
    }
}