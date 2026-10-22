use crate::{math::{Vec2, Vec3, Quat}};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        rotation: Quat,
        fov: f32,
    ) -> Self {
        Self {
            position,
            rotation,
            fov,
        }
    }
    
    pub fn NEW() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            fov: 70.0
        }
    }

    pub fn world_to_screen(
        &self,
        point: Vec3,
        screen_width: u32,
        screen_height: u32,
    ) -> Option<Vec2> {
        let relative = self.world_to_camera(point);
    
        const NEAR_PLANE: f32 = 0.1;
    
        if relative.z < NEAR_PLANE {
            return None;
        }
    
        let half_width = screen_width as f32 / 2.0;
        let half_height = screen_height as f32 / 2.0;
    
        let fov_radians = self.fov.to_radians();

        let focal_length =
            half_height / (fov_radians * 0.5).tan();

        let x =
            (relative.x / relative.z) * focal_length
            + half_width;

        let y =
            (-relative.y / relative.z) * focal_length
            + half_height;
            
        Some(Vec2::new(x, y))
    }

    pub fn world_to_camera(&self, point: Vec3) -> Vec3 {
        let relative = point - self.position;
    
        self.rotation
            .conjugate()
            .rotate_vec3(relative)
    }

}