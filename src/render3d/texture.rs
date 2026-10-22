use crate::{
    color::{Color}
};

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Texture {
    pub fn new(
        width: u32,
        height: u32,
        pixels: Vec<Color>,
    ) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn sample(
        &self,
        u: f32,
        v: f32,
    ) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
    
        let x =
            (u * self.width as f32)
            .floor() as u32;
    
        let y =
            (v * self.height as f32)
            .floor() as u32;
    
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
    
        self.pixels[
            (y * self.width + x) as usize
        ]
    }
}