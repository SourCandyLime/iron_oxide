pub trait GameContext {
    fn new() -> Self
    where
        Self: Sized;
}