use crate::{
    color::{self, Color},
    globals::{BOOT_IRON_OXIDE, BOOT_RUST_PICOCAD, DEFAULT_BOOT_SCREEN_DURATION},
    renderer::Framebuffer,
};

pub struct BootScreen {
    pub pixels: Vec<Color>,
    pub width: u32,
    pub height: u32,
    pub duration: f32,
}

impl BootScreen {
    pub fn from_bytes(image_bytes: &'static [u8], duration: f32) -> Self {
        let image = image::load_from_memory(image_bytes)
            .expect("Failed to load boot screen image")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let pixels = image
            .pixels()
            .map(|pixel| {
                let r = ((pixel[0] as f32 / 255.0) * 7.0).round() as u8;
                let g = ((pixel[1] as f32 / 255.0) * 7.0).round() as u8;
                let b = ((pixel[2] as f32 / 255.0) * 3.0).round() as u8;
                color::rgb(r, g, b)
            })
            .collect();
        Self {
            pixels,
            width,
            height,
            duration,
        }
    }
}

impl Default for BootScreen {
    fn default() -> Self {
        Self {
            pixels: Vec::new(),
            width: 0,
            height: 0,
            duration: DEFAULT_BOOT_SCREEN_DURATION,
        }
    }
}

pub struct BootState {
    pub screens: Vec<BootScreen>,
    pub current: usize,
    pub elapsed: f32,
}

impl BootState {
    pub(super) fn with_engine_screens(custom_screen: Option<BootScreen>) -> Self {
        let mut screens = vec![
            BootScreen::from_bytes(BOOT_RUST_PICOCAD, DEFAULT_BOOT_SCREEN_DURATION),
            BootScreen::from_bytes(BOOT_IRON_OXIDE, DEFAULT_BOOT_SCREEN_DURATION),
        ];
        screens.extend(custom_screen);
        Self::new(screens)
    }

    pub(super) fn render(&self, framebuffer: &mut Framebuffer) {
        if let Some(screen) = self.current_screen() {
            render_boot_screen(framebuffer, screen);
        }
    }

    pub fn new(screens: Vec<BootScreen>) -> Self {
        Self {
            screens,
            current: 0,
            elapsed: 0.0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current >= self.screens.len()
    }

    pub fn current_screen(&self) -> Option<&BootScreen> {
        self.screens.get(self.current)
    }

    pub fn update(&mut self, delta_seconds: f32) {
        if self.is_finished() {
            return;
        }
        self.elapsed += delta_seconds;
        let current_duration = self.screens[self.current].duration;
        // Advance at most one screen per frame, discarding excess time.
        if self.elapsed >= current_duration {
            self.elapsed = 0.0;
            self.current += 1;
        }
    }
}

pub fn render_boot_screen(framebuffer: &mut Framebuffer, screen: &BootScreen) {
    framebuffer.clear(color::BLACK);
    if screen.width == 0 || screen.height == 0 {
        return;
    }
    let draw_width = screen.width.min(framebuffer.width());
    let draw_height = screen.height.min(framebuffer.height());
    for y in 0..draw_height {
        for x in 0..draw_width {
            let index = (y * screen.width + x) as usize;
            let color = screen.pixels[index];
            framebuffer.set_pixel(x, y, color);
        }
    }
}
