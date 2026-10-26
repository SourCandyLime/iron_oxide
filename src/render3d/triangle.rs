use super::{Vertex, Texture};
use crate::{color::Color, render3d::Camera3D, canvas::Canvas};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub color: Color,
}

impl Triangle {
    pub fn new(vertices: [Vertex; 3], color: Color) -> Self {
        Self {
            vertices,
            color,
        }
    }

    pub fn render(
        &self,
        camera: &Camera3D,
        canvas: &mut Canvas,
        texture: Option<&Texture>,
        screen_width: u32,
        screen_height: u32,
    ) {
        let p0 = match camera.world_to_screen(
            self.vertices[0].position,
            screen_width,
            screen_height,
        ) {
            Some(point) => point,
            None => return,
        };

        let p1 = match camera.world_to_screen(
            self.vertices[1].position,
            screen_width,
            screen_height,
        ) {
            Some(point) => point,
            None => return,
        };

        let p2 = match camera.world_to_screen(
            self.vertices[2].position,
            screen_width,
            screen_height,
        ) {
            Some(point) => point,
            None => return,
        };

        let c0 = camera.world_to_camera(
            self.vertices[0].position
        );
        
        let c1 = camera.world_to_camera(
            self.vertices[1].position
        );
        
        let c2 = camera.world_to_camera(
            self.vertices[2].position
        );

        let cross =
            (p1.x - p0.x) * (p2.y - p0.y)
            - (p1.y - p0.y) * (p2.x - p0.x);

        if cross <= 0.0 {
            return;
        };
        
        let uv0 = self.vertices[0].uv;
        let uv1 = self.vertices[1].uv;
        let uv2 = self.vertices[2].uv;


        if let Some(texture) = texture {
            canvas.textured_triangle(
                p0,
                p1,
                p2,
    
                uv0,
                uv1,
                uv2,

                c0.z,
                c1.z,
                c2.z,
    
                texture,
            );
        } else {
            let points = [
                (p0.x.round() as i32, p0.y.round() as i32),
                (p1.x.round() as i32, p1.y.round() as i32),
                (p2.x.round() as i32, p2.y.round() as i32),
            ];
    
            canvas.fill_poly(
                &points,
                self.color,
            );
        }
    }

    
}