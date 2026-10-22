use crate::{renderer::Framebuffer, color::Color, math::{Vec2}, render3d::{Texture}};

pub struct Canvas<'a> {
    pub(super) framebuffer: &'a mut Framebuffer,
}

impl<'a> Canvas<'a> {
    pub fn new(framebuffer: &'a mut Framebuffer) -> Self {
        Self { framebuffer }
    }

    pub fn clear(&mut self, color: Color) {
        self.framebuffer.mut_pixels().fill(color);
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        let mut x = x1;
        let mut y = y1;
    
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
    
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
    
        let mut error = dx + dy;
    
        loop {
            if x >= 0 && y >= 0 {
                self.framebuffer.set_pixel(x as u32, y as u32, color);
            }
    
            if x == x2 && y == y2 {
                break;
            }
    
            let error2 = 2 * error;
    
            if error2 >= dy {
                error += dy;
                x += sx;
            }
    
            if error2 <= dx {
                error += dx;
                y += sy;
            }
        }
    }

    pub fn draw_hline(
        &mut self,
        x: i32,
        y: i32,
        length: u32,
        color: Color,
    ) {
        if y < 0 || y >= self.framebuffer.height() as i32 {
            return;
        }
    
        let mut start_x = x;
        let mut end_x = x + length as i32;
    
        // Clip against framebuffer bounds
        if start_x < 0 {
            start_x = 0;
        }
    
        if end_x > self.framebuffer.width() as i32 {
            end_x = self.framebuffer.width() as i32;
        }
    
        if start_x >= end_x {
            return;
        }
    
        let start = (y as u32 * self.framebuffer.width() + start_x as u32) as usize;
        let end = (y as u32 * self.framebuffer.width() + end_x as u32) as usize;
    
        self.framebuffer.mut_pixels()[start..end].fill(color);
    }

    pub fn rect(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Color,
    ) {
        if width == 0 || height == 0 {
            return;
        }
    
        if height == 1 {
            self.draw_hline(x, y, width, color);
            return;
        }
    
        if width == 1 {
            self.draw_vline(x, y, height, color);
            return;
        }
    
        let right = x + width as i32 - 1;
        let bottom = y + height as i32 - 1;
    
        self.draw_hline(x, y, width, color);
        self.draw_hline(x, bottom, width, color);
    
        if height > 2 {
            self.draw_vline(x, y + 1, height - 2, color);
            self.draw_vline(right, y + 1, height - 2, color);
        }
    }

    pub fn fill_rect(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Color,
    ) {
        if width == 0 || height == 0 {
            return;
        }
    
        let start_y = y.max(0);
        let end_y = (y + height as i32).min(self.framebuffer.height() as i32);
    
        let start_x = x.max(0);
        let end_x = (x + width as i32).min(self.framebuffer.width() as i32);
    
        if start_x >= end_x || start_y >= end_y {
            return;
        }
    
        for row in start_y..end_y {
            let start = (row as u32 * self.framebuffer.width() + start_x as u32) as usize;
            let end = (row as u32 * self.framebuffer.width() + end_x as u32) as usize;
    
            self.framebuffer.mut_pixels()[start..end].fill(color);
        }
    }

    pub fn draw_vline(
        &mut self,
        x: i32,
        y: i32,
        length: u32,
        color: Color,
    ) {
        if x < 0 || x >= self.framebuffer.width() as i32 {
            return;
        }
    
        let start_y = y.max(0);
        let end_y = (y + length as i32).min(self.framebuffer.height() as i32);
    
        if start_y >= end_y {
            return;
        }
    
        for row in start_y..end_y {
            let index = (row as u32 * self.framebuffer.width() + x as u32) as usize;
            self.framebuffer.mut_pixels()[index] = color;
        }
    }

    pub fn poly(
        &mut self,
        points: &[(i32, i32)],
        color: Color,
    ) {
        if points.len() < 2 {
            return;
        }
    
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
    
            self.draw_line(x1, y1, x2, y2, color);
        }
    }

    pub fn fill_poly(
        &mut self,
        points: &[(i32, i32)],
        color: Color,
    ) {
        if points.len() < 3 {
            return;
        }
    
        let min_y = points.iter()
            .map(|(_, y)| *y)
            .min()
            .unwrap()
            .max(0);
    
        let max_y = points.iter()
            .map(|(_, y)| *y)
            .max()
            .unwrap()
            .max(self.framebuffer.height() as i32 - 1);

        if min_y > max_y {
            return;
        }
    
        for y in min_y..=max_y {
            let mut intersections = Vec::new();
    
            for i in 0..points.len() {
                let (x1, y1) = points[i];
                let (x2, y2) = points[(i + 1) % points.len()];
    
                if y1 == y2 {
                    continue;
                }
    
                let crosses =
                    (y >= y1.min(y2)) &&
                    (y < y1.max(y2));
    
                if crosses {
                    let x = x1
                        + (y - y1) * (x2 - x1)
                            / (y2 - y1);
    
                    intersections.push(x);
                }
            }
    
            intersections.sort_unstable();
    
            for pair in intersections.chunks(2) {
                if pair.len() == 2 {
                    let start = pair[0];
                    let end = pair[1];
    
                    self.draw_hline(
                        start,
                        y,
                        (end - start + 1) as u32,
                        color,
                    );
                }
            }
        }
    }

    pub fn textured_triangle(
        &mut self,
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
    
        uv0: Vec2,
        uv1: Vec2,
        uv2: Vec2,

        z0: f32,
        z1: f32,
        z2: f32,
    
        texture: &Texture,
    ) {
        let min_x = p0.x
            .min(p1.x)
            .min(p2.x)
            .floor() as i32;
    
        let max_x = p0.x
            .max(p1.x)
            .max(p2.x)
            .ceil() as i32;
    
        let min_y = p0.y
            .min(p1.y)
            .min(p2.y)
            .floor() as i32;
    
        let max_y = p0.y
            .max(p1.y)
            .max(p2.y)
            .ceil() as i32;
    
        let min_x = min_x.max(0);
        let min_y = min_y.max(0);
    
        let max_x = max_x.min(
            self.framebuffer.width() as i32 - 1
        );
    
        let max_y = max_y.min(
            self.framebuffer.height() as i32 - 1
        );
    
        let area = edge(p0, p1, p2);
    
        if area.abs() < f32::EPSILON {
            return;
        }

        let iz0 = 1.0 / z0;
        let iz1 = 1.0 / z1;
        let iz2 = 1.0 / z2;
    
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let point = Vec2::new(
                    x as f32 + 0.5,
                    y as f32 + 0.5,
                );
    
                let w0 = edge(p1, p2, point) / area;
                let w1 = edge(p2, p0, point) / area;
                let w2 = edge(p0, p1, point) / area;
    
                if w0 >= 0.0
                    && w1 >= 0.0
                    && w2 >= 0.0
                {
                    let denominator =
                        w0 * iz0
                        + w1 * iz1
                        + w2 * iz2;
    
                    let u =
                        (
                            w0 * uv0.x * iz0
                            + w1 * uv1.x * iz1
                            + w2 * uv2.x * iz2
                        ) / denominator;
    
                    let v =
                        (
                            w0 * uv0.y * iz0
                            + w1 * uv1.y * iz1
                            + w2 * uv2.y * iz2
                        ) / denominator;
    
                    let color = texture.sample(u, v);
    
                    self.framebuffer.set_pixel(
                        x as u32,
                        y as u32,
                        color,
                    );
                }
            }
        }
    }
}

fn edge(
    a: Vec2,
    b: Vec2,
    p: Vec2,
) -> f32 {
    (p.x - a.x) * (b.y - a.y)
        - (p.y - a.y) * (b.x - a.x)
}