use super::{
    crt::{Crt, CrtConfig, CrtUniform},
    cursor::Cursor,
    frame::Frame,
    framebuffer,
    viewport::{self, ScreenUniform},
};
use crate::{app::CrtFrame, math::Vec2};
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    framebuffer: framebuffer::Framebuffer,
    crt: Crt,
    frame: Frame,
    cursor: Cursor,

    viewport_x: u32,
    viewport_y: u32,
    viewport_width: u32,
    viewport_height: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, width: u32, height: u32, frame: &CrtFrame) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .expect("Failed to find a suitable GPU adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to find device");

        println!("{:?}", adapter.get_info());

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,

            width: size.width,
            height: size.height,

            present_mode: wgpu::PresentMode::Fifo,

            desired_maximum_frame_latency: 2,

            alpha_mode: capabilities.alpha_modes[0],

            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let framebuffer = framebuffer::Framebuffer::new(width, height);

        let crt = Crt::new(&device, &queue, config.format, width, height);
        let (viewport_x, viewport_y, viewport_width, viewport_height) =
            viewport::calculate_viewport(
                PhysicalSize::new(config.width, config.height),
                PhysicalSize::new(width, height),
            );

        let screen_uniform = ScreenUniform {
            viewport_origin: [viewport_x as f32, viewport_y as f32],

            viewport_size: [viewport_width as f32, viewport_height as f32],
        };

        queue.write_buffer(
            &crt.screen_uniform_buffer,
            0,
            bytemuck::bytes_of(&screen_uniform),
        );

        let frame = Frame::new(&device, &queue, config.format, frame);
        let cursor = Cursor::new(
            &device,
            &queue,
            config.format,
            &crt.screen_uniform_buffer,
            &crt.crt_uniform_buffer,
        );

        Self {
            surface,
            device,
            queue,
            config,

            framebuffer,
            crt,
            frame,
            cursor,

            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
        }
    }

    pub fn render(&mut self) {
        self.upload_framebuffer();
        self.queue.write_buffer(
            &self.crt.crt_uniform_buffer,
            0,
            bytemuck::bytes_of(&CrtUniform::from(self.crt.config)),
        );

        self.cursor
            .update(&self.queue, self.config.width, self.config.height);

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,

            wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Occluded => return,

            wgpu::CurrentSurfaceTexture::Outdated => {
                // Later: reconfigure the surface here.
                return;
            }

            wgpu::CurrentSurfaceTexture::Lost => {
                // Later: recreate/reconfigure the surface.
                return;
            }

            wgpu::CurrentSurfaceTexture::Validation => {
                panic!("Surface validation error");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Iron Oxide Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Iron Oxide Render Pass"),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,

                    depth_slice: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),

                        store: wgpu::StoreOp::Store,
                    },
                })],

                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.crt.framebuffer_render_pipeline);

            render_pass.set_bind_group(0, &self.crt.framebuffer_bind_group, &[]);

            render_pass.set_viewport(
                self.viewport_x as f32,
                self.viewport_y as f32,
                self.viewport_width as f32,
                self.viewport_height as f32,
                0.0,
                1.0,
            );

            render_pass.draw(0..3, 0..1);

            // Draw the physical CRT enclosure over the game.
            render_pass.set_pipeline(&self.frame.render_pipeline);

            render_pass.set_bind_group(0, &self.frame.bind_group, &[]);

            // The enclosure uses the same viewport as the game framebuffer.
            render_pass.set_viewport(
                self.viewport_x as f32,
                self.viewport_y as f32,
                self.viewport_width as f32,
                self.viewport_height as f32,
                0.0,
                1.0,
            );

            render_pass.draw(0..3, 0..1);

            if self.cursor.cursor_visible {
                render_pass.set_pipeline(&self.cursor.cursor_render_pipeline);
                render_pass.set_bind_group(0, &self.cursor.cursor_bind_group, &[]);

                render_pass.set_viewport(
                    0.0,
                    0.0,
                    self.config.width as f32,
                    self.config.height as f32,
                    0.0,
                    1.0,
                );

                render_pass.draw(0..3, 0..1);
            }
        }

        let command_buffer = encoder.finish();

        self.queue.submit([command_buffer]);

        self.queue.present(output);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;

        self.surface.configure(&self.device, &self.config);

        let (viewport_x, viewport_y, viewport_width, viewport_height) =
            viewport::calculate_viewport(
                PhysicalSize::new(size.width, size.height),
                PhysicalSize::new(self.framebuffer.width(), self.framebuffer.height()),
            );

        self.viewport_x = viewport_x;
        self.viewport_y = viewport_y;
        self.viewport_width = viewport_width;
        self.viewport_height = viewport_height;

        let screen_uniform = ScreenUniform {
            viewport_origin: [self.viewport_x as f32, self.viewport_y as f32],

            viewport_size: [self.viewport_width as f32, self.viewport_height as f32],
        };

        self.queue.write_buffer(
            &self.crt.screen_uniform_buffer,
            0,
            bytemuck::bytes_of(&screen_uniform),
        );
    }

    fn upload_framebuffer(&self) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.crt.framebuffer_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.framebuffer.pixels(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.framebuffer.width()),
                rows_per_image: Some(self.framebuffer.height()),
            },
            wgpu::Extent3d {
                width: self.framebuffer.width(),
                height: self.framebuffer.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn crt(&self) -> &CrtConfig {
        &self.crt.config
    }

    pub fn crt_mut(&mut self) -> &mut CrtConfig {
        &mut self.crt.config
    }

    pub fn framebuffer(&self) -> &framebuffer::Framebuffer {
        &self.framebuffer
    }

    pub fn framebuffer_mut(&mut self) -> &mut framebuffer::Framebuffer {
        &mut self.framebuffer
    }

    pub fn window_to_framebuffer(&self, x: f64, y: f64) -> Option<Vec2> {
        viewport::window_to_framebuffer(
            (
                self.viewport_x,
                self.viewport_y,
                self.viewport_width,
                self.viewport_height,
            ),
            &self.framebuffer,
            self.crt.config.curvature,
            x,
            y,
        )
    }

    pub fn set_cursor_position(&mut self, position: Vec2) {
        self.cursor.cursor_position = position;
    }

    pub fn hide_cursor(&mut self) {
        self.cursor.cursor_visible = false;
    }

    pub fn show_cursor(&mut self) {
        self.cursor.cursor_visible = true;
    }
}
