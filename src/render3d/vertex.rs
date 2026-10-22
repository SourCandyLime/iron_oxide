use crate::{
    math::{Vec2, Vec3}
};

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2
}

impl Vertex {
    pub const fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            uv: Vec2::new(u, v)
        }
    }
}