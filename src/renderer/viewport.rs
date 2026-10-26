use crate::math::Vec2;
use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ScreenUniform {
    pub(super) viewport_origin: [f32; 2],
    pub(super) viewport_size: [f32; 2],
}

pub(super) fn calculate_viewport(
    window_size: PhysicalSize<u32>,
    framebuffer_size: PhysicalSize<u32>,
) -> (u32, u32, u32, u32) {
    let scale_x = window_size.width / framebuffer_size.width;
    let scale_y = window_size.height / framebuffer_size.height;

    let scale = scale_x.min(scale_y).max(1);

    let viewport_width = framebuffer_size.width * scale;
    let viewport_height = framebuffer_size.height * scale;

    let viewport_x = (window_size.width - viewport_width) / 2;
    let viewport_y = (window_size.height - viewport_height) / 2;

    (viewport_x, viewport_y, viewport_width, viewport_height)
}

pub(super) fn window_to_framebuffer(
    viewport: (u32, u32, u32, u32),
    framebuffer: &super::framebuffer::Framebuffer,
    curvature: f32,
    x: f64,
    y: f64,
) -> Option<Vec2> {
    let x = x as f32;
    let y = y as f32;

    let vx = viewport.0 as f32;
    let vy = viewport.1 as f32;
    let vw = viewport.2 as f32;
    let vh = viewport.3 as f32;

    if x < vx || y < vy || x >= vx + vw || y >= vy + vh {
        return None;
    }

    let normalized_x = (x - vx) / vw;

    let normalized_y = (y - vy) / vh;

    let cx = normalized_x * 2.0 - 1.0;
    let cy = normalized_y * 2.0 - 1.0;

    let d = 1.0 + (0.12 * curvature * (cx.powi(2) + cy.powi(2)));

    let u = (cx * d + 1.0) / 2.0;
    let v = (cy * d + 1.0) / 2.0;

    if (u < 0.0 || u >= 1.0) || (v < 0.0 || v >= 1.0) {
        return None;
    }

    Some(Vec2::new(
        u * framebuffer.width() as f32,
        v * framebuffer.height() as f32,
    ))
}
