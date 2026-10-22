use crate::{color::Color};

pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn mut_pixels(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = (y * self.width + x) as usize;
        self.pixels[index] = color;
    }

    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color);
    }

}

