use crate::*;

pub const FRAMEBUFFER_WIDTH: u32 = 480;
pub const FRAMEBUFFER_HEIGHT: u32 = 270;

pub const FRAMEBUFFER_CENTER_Y: u32 = FRAMEBUFFER_HEIGHT / 2;
pub const FRAMEBUFFER_CENTER_X: u32 = FRAMEBUFFER_WIDTH / 2;

pub const FRAMEBUFFER_CENTER: Vec2 = Vec2::new(FRAMEBUFFER_CENTER_X as f32, FRAMEBUFFER_CENTER_Y as f32);

pub const DEFAULT_WINDOW_TITLE: &str = "Iron Oxide";

pub const NEAR_PLANE: f32 = 0.1;

pub const DEFAULT_BOOT_SCREEN_DURATION: f32 = 1.5;

pub const BOOT_RUST_PICOCAD: &[u8] =
    include_bytes!("assets/BootScreen2.png");

pub const BOOT_IRON_OXIDE: &[u8] =
    include_bytes!("assets/BootScreen1.png");

pub const ICON_IRON_OXIDE: &[u8] =
    include_bytes!("assets/IronOxide.png");

pub const  CRT_FRAME_BLANK_BYTES: &[u8] =
    include_bytes!("assets/frame.png");

pub const CURSOR_BYTES: &[u8] =
    include_bytes!("assets/Cursor.png");