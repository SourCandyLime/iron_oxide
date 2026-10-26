use super::AppConfig;
use crate::{globals::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH}, renderer::Renderer, CrtFrame, CRT_FRAME_BLANK_BYTES};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Icon, Window, WindowAttributes},
};

/// The window and its renderer are always initialized together.
pub(super) struct AppWindow {
    pub handle: Arc<Window>,
    pub renderer: Renderer,
}

impl AppWindow {
    pub fn new(event_loop: &ActiveEventLoop, config: &AppConfig) -> Self {
        let mut attributes = WindowAttributes::default()
            .with_title(&config.title)
            .with_min_inner_size(PhysicalSize::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT))
            .with_visible(false);
        if let Some(icon) = &config.icon {
            let icon = Icon::from_rgba(icon.rgba.clone(), icon.width, icon.height)
                .expect("Invalid window icon");
            attributes = attributes.with_window_icon(Some(icon));
        }
        let handle = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window"),
        );
        
        
        let mut frame= &CrtFrame::from_bytes(CRT_FRAME_BLANK_BYTES);
        
        if let Some(f) = &config.frame {
            frame = config.frame.as_ref().unwrap();
        }
            
        let renderer = pollster::block_on(Renderer::new(
            handle.clone(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
            frame
        ));

        Self { handle, renderer }
    }

    pub fn toggle_fullscreen(&self) {
        let fullscreen = if self.handle.fullscreen().is_some() {
            None
        } else {
            Some(Fullscreen::Borderless(None))
        };
        self.handle.set_fullscreen(fullscreen);
    }
}
