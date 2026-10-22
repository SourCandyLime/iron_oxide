use crate::{
    canvas::Canvas,
    math::{Vec3, Quat},
};

use super::{
    Camera,
    Mesh,
    Texture,
};

pub struct Model {
    pub position: Vec3,
    pub rotation: Quat,
    pub meshes: Vec<Mesh>,
    pub texture: Option<Texture>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>, texture: Option<Texture>) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            meshes,
            texture,
        }
    }

    pub fn BLANK() -> Self {
        Self::new(Vec::new(), None)
    }
 
    pub fn local_pos_to_world_pos(&self, pos: Vec3) -> Vec3 {
        self.rotation.rotate_vec3(pos) + self.position
    }
    
    pub fn local_rot_to_world_rot(&self, rot: Quat) -> Quat {
        self.rotation * rot
    } 

    pub fn render(
        &self,
        camera: &Camera,
        canvas: &mut Canvas,
        screen_width: u32,
        screen_height: u32,
    ) {
        for mesh in &self.meshes {
            let transformed = Mesh {
                position: self.local_pos_to_world_pos(mesh.position),
                rotation: self.local_rot_to_world_rot(mesh.rotation),
                triangles: mesh.triangles.clone()
            };
            transformed.render(
                camera,
                canvas,
                self.texture.as_ref(),
                screen_width,
                screen_height,
            );
        }
    }
}