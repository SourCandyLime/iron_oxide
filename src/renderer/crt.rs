use super::viewport::ScreenUniform;
use bytemuck::{Pod, Zeroable};

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
pub(super) struct CrtUniform {
    controls: [f32; 4], // enabled, phosphor, scanlines, curvature
    display: [f32; 4],  // brightness, unused, unused, unused
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

/// GPU resources for framebuffer presentation and CRT post-processing.
pub(super) struct Crt {
    pub(super) framebuffer_texture: wgpu::Texture,
    pub(super) framebuffer_bind_group: wgpu::BindGroup,
    pub(super) framebuffer_render_pipeline: wgpu::RenderPipeline,
    pub(super) screen_uniform_buffer: wgpu::Buffer,
    pub(super) crt_uniform_buffer: wgpu::Buffer,
    pub(super) config: CrtConfig,
}

impl Crt {
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
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

            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,

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
        queue.write_buffer(
            &crt_uniform_buffer,
            0,
            bytemuck::bytes_of(&CrtUniform::from(crt)),
        );

        let framebuffer_view =
            framebuffer_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                ],
            });

        let framebuffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iron Oxide Framebuffer Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("framebuffer.wgsl").into()),
        });

        let framebuffer_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Iron Oxide Framebuffer Pipeline Layout"),

                bind_group_layouts: &[Some(&framebuffer_bind_group_layout)],

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
                        format,
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

        Self {
            framebuffer_texture,
            framebuffer_bind_group,
            framebuffer_render_pipeline,
            screen_uniform_buffer,
            crt_uniform_buffer,
            config: crt,
        }
    }
}
