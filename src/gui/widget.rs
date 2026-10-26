use crate::{
    Canvas,
    Input
};

pub trait Widget {
    fn new(x: i32, y: i32, width: u32, height: u32, text: String, id: String, colors: &[u8]) -> Self
    where Self: Sized;

    fn id(&self) -> &String;
    fn update(&mut self, input: &Input) {}
    fn draw(&self, canvas: &mut Canvas) {}

    fn interactable(&self) -> bool { false }

    fn down(&self) -> bool { false }

    fn pressed(&self) -> bool { false }

    fn released(&self) -> bool { false }

    fn clicked(&self) -> bool { false }
}