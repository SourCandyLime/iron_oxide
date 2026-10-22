use crate::{
    canvas::Canvas, math::{Quat, Vec3}, render3d::Texture
};

use super::{
    Camera,
    Triangle,
    Vertex
};

pub struct Mesh {
    pub position: Vec3,
    pub rotation: Quat,
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(
        position: Vec3,
        rotation: Quat,
        triangles: Vec<Triangle>
    ) -> Self {
        Self {
            position,
            rotation,
            triangles
        }
    }

    pub fn from_triangles(triangles: Vec<Triangle>) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            triangles,
        }
    }

    pub fn local_to_world(&self, point: Vec3) -> Vec3 {
        self.rotation.rotate_vec3(point) + self.position
    }

    pub fn render(
        &self,
        camera: &Camera,
        canvas: &mut Canvas,
        texture: Option<&Texture>,
        screen_width: u32,
        screen_height: u32,
    ) {
        for triangle in &self.triangles {
            let transformed = Triangle {
                vertices: triangle.vertices.map(|vertex| {
                    Vertex {
                        position: self.local_to_world(vertex.position),
                        uv: vertex.uv,
                    }
                }),
            
                color: triangle.color,
            };
    
            transformed.render(
                camera,
                canvas,
                texture,
                screen_width,
                screen_height,
            );
        }
    }
}