pub struct CrtFrame {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl CrtFrame {
    pub fn from_bytes(image_bytes: &'static [u8]) -> Self {
        let image = image::load_from_memory(image_bytes)
            .expect("Failed to load boot screen image")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let pixels = image.into_raw();
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}
impl Default for CrtFrame {
    fn default() -> Self {
        Self {
            pixels: Vec::new(),
            width: 0,
            height: 0,
        }
    }
}

