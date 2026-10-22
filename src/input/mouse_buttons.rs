#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub(crate) fn from_winit(
        button: winit::event::MouseButton,
    ) -> Option<Self> {
        match button {
            winit::event::MouseButton::Left =>
                Some(Self::Left),

            winit::event::MouseButton::Right =>
                Some(Self::Right),

            winit::event::MouseButton::Middle =>
                Some(Self::Middle),

            _ => None,
        }
    }
}