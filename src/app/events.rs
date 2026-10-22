use super::{App, BootState, app::AppState, window::AppWindow};
use crate::{game::Game, input::{Key, MouseButton}};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

impl<G: Game> App<G> {
    fn initialize(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut window = AppWindow::new(event_loop, &self.config);
        let boot = BootState::with_engine_screens(self.config.boot_screen.take());

        // Fill and present the hidden window before revealing it.
        boot.render(window.renderer.framebuffer_mut());
        window.renderer.render();
        window.handle.set_visible(true);
        self.state = AppState::Boot(boot);
        self.window = Some(window);
        self.time.reset();
    }

    fn handle_keyboard(&mut self, event: KeyEvent) {
        if event.physical_key == PhysicalKey::Code(KeyCode::F11)
            && event.state == ElementState::Pressed
            && !event.repeat
        {
            if let Some(window) = &self.window {
                window.toggle_fullscreen();
            }
        }
        if let PhysicalKey::Code(code) = event.physical_key {
            if let Some(key) = Key::from_winit(code) {
                self.input.key_event(key, event.state);
            }
        }
    }
}

impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.initialize(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(size) => {
                if let Some(window) = &mut self.window {
                    window.renderer.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard(event),
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) =
                    MouseButton::from_winit(button)
                {
                    self.input.mouse_button_event(button, state);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {

                let mouse_position = self.window.as_ref().unwrap().renderer.window_to_framebuffer(position.x, position.y);

                self.input.set_mouse_position(
                    mouse_position,
                );
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.handle.request_redraw();
        }
    }
}
