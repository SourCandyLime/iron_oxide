use super::{window::AppWindow, AppConfig, AppIcon, BootScreen, BootState, CrtFrame};
use crate::{game::Game, input::Input, time::Time, CrtConfig, GameAction};
use std::path::Path;
use winit::event_loop::EventLoop;

pub(super) enum AppState {
    Uninitialized,
    Boot(BootState),
    Running,
}

/// Owns the game and drives its boot, update, and render lifecycle.
pub struct App<G: Game> {
    pub(super) game: G,
    pub(super) window: Option<AppWindow>,
    pub(super) input: Input,
    pub(super) time: Time,
    pub(super) config: AppConfig,
    pub(super) state: AppState,
    pub(super) crt_config: CrtConfig,
    pub(super) exit: bool,
}

impl<G: Game> App<G> {
    pub fn new(game: G) -> Self {
        Self {
            game,
            window: None,
            input: Input::new(),
            time: Time::new(),
            config: AppConfig::default().icon(crate::globals::ICON_IRON_OXIDE),
            state: AppState::Uninitialized,
            crt_config: CrtConfig::default(),
            exit: false,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.config.title = title.to_string();
        self
    }

    pub fn icon_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.icon = Some(AppIcon::from_path(path));
        self
    }
    pub fn icon_bytes(mut self, data: &[u8]) -> Self {
        self.config.icon = Some(AppIcon::from_bytes(data));
        self
    }

    /// Appends a custom screen after the two built-in engine boot screens.
    pub fn boot_screen(mut self, image_bytes: &'static [u8], duration: f32) -> Self {
        self.config.boot_screen = Some(BootScreen::from_bytes(image_bytes, duration));
        self
    }

    pub fn crt_frame(mut self, image_bytes: &'static [u8]) -> Self {
        self.config.frame = Some(CrtFrame::from_bytes(image_bytes));
        self
    }

    pub fn crt(mut self, config: CrtConfig) -> Self {
        self.crt_config = config;
        self
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop
            .run_app(&mut self)
            .expect("Iron Oxide failed while running");
    }

    pub(super) fn redraw(&mut self) {
        self.time.update();
        let Some(window) = &mut self.window else {
            return;
        };
        let renderer = &mut window.renderer;
        match &mut self.state {
            AppState::Uninitialized => return,
            AppState::Boot(boot) => {
                boot.update(self.time.delta_seconds());

                boot.render(renderer.framebuffer_mut());
                renderer.render();

                // Present the final boot frame before starting the game.
                // The first game update and render happen on the next redraw.
                if boot.is_finished() {
                    self.game.start();
                    self.state = AppState::Running;
                }
            }
            AppState::Running => {
                if self.game.update(&self.time, &self.input) == GameAction::Exit {
                    self.exit = true;
                    return;
                }
                self.game.render(renderer.framebuffer_mut());
                renderer.render();
            }
        }
        self.input.end_frame();
        window.handle.request_redraw();
    }
}
