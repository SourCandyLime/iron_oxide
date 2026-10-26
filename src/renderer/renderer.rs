use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};
use bytemuck::{Pod, Zeroable};

use crate::{
    renderer::{framebuffer, },
    math::{Vec2, },
    globals::*,
};
use crate::app::CrtFrame;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScreenUniform {
    viewport_origin: [f32; 2],
    viewport_size: [f32; 2],
}


/// Game-facing CRT controls. All strengths are clamped to 0.0..=1.0
/// when sent to the GPU. Curvature is intentionally subtle.
#[derive(Debug, Clone, Copy)]
pub struct CrtConfig {
    pub enabled: bool,
    pub phosphor_strength: f32,
    pub scanline_strength: f32,
    pub curvature: f32,
    pub brightness: f32,
}

impl Default for CrtConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            phosphor_strength: 1.0,
            scanline_strength: 0.6,
            curvature: 1.0,
            brightness: 1.2,
        }
    }
}

// Two vec4<f32> slots: simple, predictable WGSL uniform alignment.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CrtUniform {
    controls: [f32; 4], // enabled, phosphor, scanlines, curvature
    display: [f32; 4],  // brightness, unused, unused, unused
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CursorUniform {
    position: [f32; 2],
    size: [f32; 2],
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

impl From<CrtConfig> for CrtUniform {
    fn from(value: CrtConfig) -> Self {
        Self {
            controls: [
                if value.enabled { 1.0 } else { 0.0 },
                value.phosphor_strength.clamp(0.0, 1.0),
                value.scanline_strength.clamp(0.0, 1.0),
                value.curvature.clamp(0.0, 1.0),
            ],
            display: [value.brightness.clamp(0.0, 3.0), 0.0, 0.0, 0.0],
        }
    }
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
    crt_uniform_buffer: wgpu::Buffer,

    crt: CrtConfig,
    crt_frame: wgpu::Texture,
    crt_frame_bind_group: wgpu::BindGroup,
    crt_frame_render_pipeline: wgpu::RenderPipeline,

    cursor_texture: wgpu::Texture,
    cursor_bind_group: wgpu::BindGroup,
    cursor_render_pipeline: wgpu::RenderPipeline,
    cursor_uniform_buffer: wgpu::Buffer,
    cursor_position: Vec2,
    cursor_visible: bool,

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

        let crt = CrtConfig::default();
        let crt_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iron Oxide CRT Uniform Buffer"),
            size: size_of::<CrtUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&crt_uniform_buffer, 0, bytemuck::bytes_of(&CrtUniform::from(crt)));

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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: crt_uniform_buffer.as_entire_binding(),
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

        let frame_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Iron Oxide CRT Frame"),
            size: wgpu::Extent3d {
                width: frame.width(),
                height: frame.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &frame_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            frame.pixels(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame.width() * 4),
                rows_per_image: Some(frame.height()),
            },
            wgpu::Extent3d {
                width: frame.width(),
                height: frame.height(),
                depth_or_array_layers: 1,
            },
        );

        let frame_view = frame_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let frame_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("CRT Frame Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("CRT Frame Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering
                        ),
                        count: None,
                    },
                ],
            });

        let crt_frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CRT Frame Bind Group"),
            layout: &frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&frame_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&frame_sampler),
                },
            ],
        });

        let frame_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CRT Frame Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("crt_frame.wgsl").into()
            ),
        });

        let frame_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("CRT Frame Pipeline Layout"),
                bind_group_layouts: &[Some(&frame_bind_group_layout)],
                immediate_size: 0,
            });

        let crt_frame_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("CRT Frame Render Pipeline"),
                layout: Some(&frame_pipeline_layout),

                vertex: wgpu::VertexState {
                    module: &frame_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },

                fragment: Some(wgpu::FragmentState {
                    module: &frame_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let cursor_image = image::load_from_memory(CURSOR_BYTES)
            .expect("Failed to decode cursor PNG")
            .into_rgba8();

        let cursor_width = cursor_image.width();
        let cursor_height = cursor_image.height();

        let cursor_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Iron Oxide Cursor Texture"),
            size: wgpu::Extent3d {
                width: cursor_width,
                height: cursor_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &cursor_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            cursor_image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(cursor_width * 4),
                rows_per_image: Some(cursor_height),
            },
            wgpu::Extent3d {
                width: cursor_width,
                height: cursor_height,
                depth_or_array_layers: 1,
            },
        );

        let cursor_view = cursor_texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let cursor_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Iron Oxide Cursor Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let cursor_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Iron Oxide Cursor Uniform Buffer"),
            size: size_of::<CursorUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cursor_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Iron Oxide Cursor Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let cursor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Iron Oxide Cursor Bind Group"),
            layout: &cursor_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&cursor_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&cursor_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: cursor_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: screen_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: crt_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let cursor_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iron Oxide Cursor Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("cursor.wgsl").into()
            ),
        });

        let cursor_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Iron Oxide Cursor Pipeline Layout"),
                bind_group_layouts: &[Some(&cursor_bind_group_layout)],
                immediate_size: 0,
            });

        let cursor_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Iron Oxide Cursor Render Pipeline"),
                layout: Some(&cursor_pipeline_layout),

                vertex: wgpu::VertexState {
                    module: &cursor_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },

                fragment: Some(wgpu::FragmentState {
                    module: &cursor_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            crt_uniform_buffer,
            crt,
            crt_frame: frame_texture,
            crt_frame_bind_group,
            crt_frame_render_pipeline,

            cursor_texture,
            cursor_bind_group,
            cursor_render_pipeline,
            cursor_uniform_buffer,
            cursor_position: Vec2::new(0.0, 0.0),
            cursor_visible: true,

            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
        }
    }

    pub fn render(&mut self) {

        self.upload_framebuffer();
        self.queue.write_buffer(
            &self.crt_uniform_buffer,
            0,
            bytemuck::bytes_of(&CrtUniform::from(self.crt)),
        );

        let cursor_uniform = CursorUniform {
            position: [
                self.cursor_position.x,
                self.cursor_position.y,
            ],
            size: [18.0, 22.0],
            screen_size: [
                self.config.width as f32,
                self.config.height as f32,
            ],
            _padding: [0.0; 2],
        };

        self.queue.write_buffer(
            &self.cursor_uniform_buffer,
            0,
            bytemuck::bytes_of(&cursor_uniform),
        );

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

            // Draw the physical CRT enclosure over the game.
            render_pass.set_pipeline(&self.crt_frame_render_pipeline);

            render_pass.set_bind_group(
                0,
                &self.crt_frame_bind_group,
                &[],
            );

            // Unlike the game framebuffer, the frame covers the whole window.
            render_pass.set_viewport(
                self.viewport_x as f32,
                self.viewport_y as f32,
                self.viewport_width as f32,
                self.viewport_height as f32,
                0.0,
                1.0,
            );

            render_pass.draw(0..3, 0..1);

            if self.cursor_visible {
                render_pass.set_pipeline(&self.cursor_render_pipeline);
                render_pass.set_bind_group(0, &self.cursor_bind_group, &[]);

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

    pub fn crt(&self) -> &CrtConfig {
        &self.crt
    }

    pub fn crt_mut(&mut self) -> &mut CrtConfig {
        &mut self.crt
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

        let cx = normalized_x * 2.0 - 1.0;
        let cy = normalized_y * 2.0 - 1.0;

        let d = 1.0 + (0.12 * self.crt.curvature * (cx.powi(2) + cy.powi(2)));

        let u = (cx * d + 1.0) / 2.0;
        let v = (cy * d + 1.0) / 2.0;

        if (u < 0.0 || u >= 1.0) || (v < 0.0 || v >= 1.0) {
            return None;
        }

        Some(Vec2::new(
            u
                * self.framebuffer.width() as f32,

            v
                * self.framebuffer.height() as f32,
        ))
    }

    pub fn set_cursor_position(&mut self, position: Vec2) {
        self.cursor_position = position;
    }

    pub fn hide_cursor(&mut self) {
        self.cursor_visible = false;
    }

    pub fn show_cursor(&mut self) {
        self.cursor_visible = true;
    }
}
