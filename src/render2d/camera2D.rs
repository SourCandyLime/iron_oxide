use std::ops::Rem;
use crate::{math::{Vec2}, FRAMEBUFFER_CENTER};

pub struct Camera2D {
    position: Vec2,
    rotation: f32,
    zoom: f32,
}

impl Camera2D {
    pub fn new(position: Vec2, rotation: f32, zoom: f32) -> Self {
        let rotation = rotation.rem_euclid(std::f32::consts::TAU);
        let zoom = zoom.max(0.01);

        Self { position, rotation, zoom }
    }

    pub fn world_to_screen(&self, position: Vec2) -> Vec2 {
        let mut result = position - self.position;
        result = result * self.zoom;

        let (sin, cos) = (-self.rotation).sin_cos();

        let x = result.x * cos - result.y * sin;
        let y = result.x * sin + result.y * cos;

        result = Vec2::new(x, y);


        result + FRAMEBUFFER_CENTER
    }

    pub fn screen_to_world(&self, position: Vec2) -> Vec2 {
        let mut result = position - FRAMEBUFFER_CENTER;

        let (sin, cos) = (self.rotation).sin_cos();

        let x = result.x * cos - result.y * sin;
        let y = result.x * sin + result.y * cos;
        result = Vec2::new(x, y);
        result / self.zoom + self.position
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.max(0.01);
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation.rem_euclid(std::f32::consts::TAU);
    }

    pub fn rotation_degrees(&mut self, rotation: f32) {
        self.rotation = rotation.to_radians();
    }
    pub fn rotation_radians(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            zoom: 1.0,
        }
    }
}
