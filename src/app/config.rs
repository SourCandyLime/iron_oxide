use super::{BootScreen, CrtFrame};
use crate::globals::DEFAULT_WINDOW_TITLE;
use std::path::Path;

pub struct AppIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl AppIcon {
    pub(super) fn from_path(path: impl AsRef<Path>) -> Self {
        let image = image::open(path)
            .expect("Failed to load window icon (Path)")
            .into_rgba8();
        let (width, height) = image.dimensions();
        Self {
            rgba: image.into_raw(),
            width,
            height,
        }
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Self {
        let image = image::load_from_memory(bytes)
            .expect("Failed to load window icon (Bytes)")
            .into_rgba8();
        let (width, height) = image.dimensions();
        Self {
            rgba: image.into_raw(),
            width,
            height,
        }
    }
}

pub struct AppConfig {
    pub title: String,
    pub icon: Option<AppIcon>,
    pub boot_screen: Option<BootScreen>,
    pub frame: Option<CrtFrame>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: DEFAULT_WINDOW_TITLE.to_string(),
            icon: None,
            boot_screen: None,
            frame: None,
        }
    }
}

impl AppConfig {
    pub fn icon(mut self, data: &[u8]) -> Self {
        self.icon = Some(AppIcon::from_bytes(data));
        self
    }
}