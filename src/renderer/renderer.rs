use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};
use bytemuck::{Pod, Zeroable};

use crate::{
    renderer::{framebuffer, },
    math::{Vec2, },
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScreenUniform {
    viewport_origin: [f32; 2],
    viewport_size: [f32; 2],
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    framebuffer: framebuffer::Framebuffer,
    framebuffer_texture: wgpu::Texture,
    framebuffer_bind_group: wgpu::BindGroup,
    framebuffer_render_pipeline: wgpu::RenderPipeline,

    screen_uniform_buffer: wgpu::Buffer,

    viewport_x: u32,
    viewport_y: u32,
    viewport_width: u32,
    viewport_height: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
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

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default())
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

        let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Iron Oxide Framebuffer Texture"),
        
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        
            mip_level_count: 1,
            sample_count: 1,
        
            dimension: wgpu::TextureDimension::D2,
        
            format: wgpu::TextureFormat::R8Uint,
        
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
        
            view_formats: &[],
        });

        let screen_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iron Oxide Screen Uniform Buffer"),
            size: size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let framebuffer_view = framebuffer_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let framebuffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Iron Oxide Framebuffer Bind Group Layout"),
        
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ]
            });

        let framebuffer_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Iron Oxide Framebuffer Bind Group"),

                layout: &framebuffer_bind_group_layout,

                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&framebuffer_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: screen_uniform_buffer.as_entire_binding(),
                    },
                ]
            });

        let shader = 
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Iron Oxide Framebuffer Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("framebuffer.wgsl").into()
                ),
            });

        let framebuffer_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Iron Oxide Framebuffer Pipeline Layout"),
        
                bind_group_layouts: &[
                    Some(&framebuffer_bind_group_layout),
                ],
        
                immediate_size: 0,
            });

        let framebuffer_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Iron Oxide Framebuffer Render Pipeline"),
        
                layout: Some(&framebuffer_pipeline_layout),
        
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
        
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
        
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
        
                    compilation_options: Default::default(),
                }),
        
                primitive: wgpu::PrimitiveState::default(),
        
                depth_stencil: None,
        
                multisample: wgpu::MultisampleState::default(),
        
                multiview_mask: None,
        
                cache: None,
            });

        let (viewport_x, viewport_y, viewport_width, viewport_height) =
            Self::calculate_viewport(
                PhysicalSize::new(config.width, config.height),
                PhysicalSize::new(width, height),
            );

        let screen_uniform = ScreenUniform {
            viewport_origin: [
                viewport_x as f32,
                viewport_y as f32,
            ],
        
            viewport_size: [
                viewport_width as f32,
                viewport_height as f32,
            ],
        };



        queue.write_buffer(
                &screen_uniform_buffer,
                0,
                bytemuck::bytes_of(&screen_uniform),
            );

        Self {
            surface,
            device,
            queue,
            config,
            
            framebuffer,
            framebuffer_texture,
            framebuffer_bind_group,
            framebuffer_render_pipeline,
            
            screen_uniform_buffer,

            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
        }
    }

    pub fn render(&mut self) {

        self.upload_framebuffer();

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

            render_pass.set_pipeline(&self.framebuffer_render_pipeline);

            render_pass.set_bind_group(
                0,
                &self.framebuffer_bind_group,
                &[],
            );

            render_pass.set_viewport(
                self.viewport_x as f32, 
                self.viewport_y as f32,
                self.viewport_width as f32,
                self.viewport_height as f32,
                0.0,
                1.0
            );
        
            render_pass.draw(0..3, 0..1);
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
            Self::calculate_viewport(
                PhysicalSize::new(size.width, size.height),
                PhysicalSize::new(self.framebuffer.width(), self.framebuffer.height()),
            );

        self.viewport_x = viewport_x;
        self.viewport_y = viewport_y;
        self.viewport_width = viewport_width;
        self.viewport_height = viewport_height;

        let screen_uniform = ScreenUniform {
            viewport_origin: [
                self.viewport_x as f32,
                self.viewport_y as f32,
            ],
        
            viewport_size: [
                self.viewport_width as f32,
                self.viewport_height as f32,
            ],
        };
        
        self.queue.write_buffer(
            &self.screen_uniform_buffer,
            0,
            bytemuck::bytes_of(&screen_uniform),
        );
    }

    fn upload_framebuffer(&self) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.framebuffer_texture,
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

    fn calculate_viewport(window_size: PhysicalSize<u32>, framebuffer_size: PhysicalSize<u32>) -> (u32, u32, u32, u32) {
        let scale_x = window_size.width / framebuffer_size.width;
        let scale_y = window_size.height / framebuffer_size.height;
        
        let scale = scale_x.min(scale_y).max(1);
        
        let viewport_width = framebuffer_size.width * scale;
        let viewport_height = framebuffer_size.height * scale;
        
        let viewport_x = (window_size.width - viewport_width) / 2;
        let viewport_y = (window_size.height - viewport_height) / 2;

        (viewport_x, viewport_y, viewport_width, viewport_height)
    }

    pub fn framebuffer(&self) -> &framebuffer::Framebuffer {
        &self.framebuffer
    }
    
    pub fn framebuffer_mut(&mut self) -> &mut framebuffer::Framebuffer {
        &mut self.framebuffer
    }

    pub fn window_to_framebuffer(
        &self,
        x: f64,
        y: f64,
    ) -> Option<Vec2> {
        let x = x as f32;
        let y = y as f32;

        let vx = self.viewport_x as f32;
        let vy = self.viewport_y as f32;
        let vw = self.viewport_width as f32;
        let vh = self.viewport_height as f32;

        if x < vx
            || y < vy
            || x >= vx + vw
            || y >= vy + vh
        {
            return None;
        }

        let normalized_x =
            (x - vx) / vw;

        let normalized_y =
            (y - vy) / vh;

        Some(Vec2::new(
            normalized_x
                * self.framebuffer.width() as f32,

            normalized_y
                * self.framebuffer.height() as f32,
        ))
    }
}