use crate::{color, Canvas, Input, MouseButton};
use crate::gui::Widget;

pub struct Button {
    x: i32,
    y: i32,
    width: u32,
    height: u32,

    id: String,
    text: String,
    color: u8,
    color_normal: u8,
    color_hover: u8,
    color_down: u8,

    down: bool,
    pressed: bool,
    released: bool,
    clicked: bool,

    hover: bool,
}


impl Widget for Button {
    fn new(x: i32, y: i32, width: u32, height: u32, text: String, id: String, colors: &[u8]) -> Button {
        Self {
            x,
            y,
            width,
            height,

            id,

            text,
            color: 0,
            color_normal: colors[0],
            color_hover: colors[1],
            color_down: colors[2],

            down: false,
            pressed: false,
            released: false,
            clicked: false,

            hover: false,
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        canvas.fill_rect(
            self.x,
            self.y,
            self.width,
            self.height,
            self.color,
        );

        let text_x = (self.x as f32 + ((self.width / 2) as f32 - (((self.text.len() * 6) / 2) - 1) as f32)).floor() as i32;
        let text_y = (self.y as f32 + ((self.height / 2) as f32 - (3.5))).floor() as i32;

        canvas.text(
            text_x,
            text_y,
            self.text.as_str(),
            color::BLACK,
        );
    }

    fn update(&mut self, input: &Input) {
        self.hover =
            input.mouse_position()
                .map(|mouse| {
                    mouse.x >= self.x as f32
                        && mouse.x < (self.x + self.width as i32) as f32
                        && mouse.y >= self.y as f32
                        && mouse.y < (self.y + self.height as i32) as f32
                })
                .unwrap_or(false);

        self.down = input.mouse_down(MouseButton::Left) && (self.hover);
        self.pressed = input.mouse_pressed(MouseButton::Left) && self.hover;
        self.released = input.mouse_released(MouseButton::Left) && self.down;
        self.clicked = input.mouse_released(MouseButton::Left) && self.hover;

        if self.down {
            self.color = self.color_down;
        } else if self.hover {
            self.color = self.color_hover;
        } else {
            self.color = self.color_normal;
        }
    }

    fn interactable(&self) -> bool { true }
    fn down(&self) -> bool { self.down }
    fn pressed(&self) -> bool { self.pressed }
    fn released(&self) -> bool { self.released }
    fn clicked(&self) -> bool { self.clicked }
    fn id(&self) -> &String { &self.id }
}