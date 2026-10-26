use super::{App, BootState, app::AppState, window::AppWindow};
use crate::{game::Game, input::{Key, MouseButton}, math::Vec2};
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
        *window.renderer.crt_mut() = self.crt_config;
        let boot = BootState::with_engine_screens(self.config.boot_screen.take());

        // Fill and present the hidden window before revealing it.
        boot.render(window.renderer.framebuffer_mut());
        window.renderer.render();

        window.handle.set_cursor_visible(false);
        window.handle.set_visible(true);

        self.state = AppState::Boot(boot);
        self.time.reset();
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
            WindowEvent::RedrawRequested => {
                self.redraw();
                if self.exit {
                    event_loop.exit();
                    return;
                }
            },
            WindowEvent::Resized(size) => {
                if let Some(window) = &mut self.window {
                    window.renderer.resize(size);
                }
            },
            WindowEvent::CursorLeft { device_id: _ } => self.window.as_mut().unwrap().renderer.hide_cursor(),
            WindowEvent::CursorEntered { device_id: _ } => self.window.as_mut().unwrap().renderer.show_cursor(),
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard(event),
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(button) =
                    MouseButton::from_winit(button)
                {
                    self.input.mouse_button_event(button, state);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {

                self.window.as_mut().unwrap().renderer.set_cursor_position(Vec2::new(position.x as f32, position.y as f32));

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
