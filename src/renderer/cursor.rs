use crate::{globals::CURSOR_BYTES, math::Vec2};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CursorUniform {
    position: [f32; 2],
    size: [f32; 2],
    screen_size: [f32; 2],
    _padding: [f32; 2],
}

pub(super) struct Cursor {
    _texture: wgpu::Texture,
    pub(super) cursor_bind_group: wgpu::BindGroup,
    pub(super) cursor_render_pipeline: wgpu::RenderPipeline,
    cursor_uniform_buffer: wgpu::Buffer,
    pub(super) cursor_position: Vec2,
    pub(super) cursor_visible: bool,
}

impl Cursor {
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        screen_uniform_buffer: &wgpu::Buffer,
        crt_uniform_buffer: &wgpu::Buffer,
    ) -> Self {
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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

        let cursor_view = cursor_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
            source: wgpu::ShaderSource::Wgsl(include_str!("cursor.wgsl").into()),
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
                        format,
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
            _texture: cursor_texture,
            cursor_bind_group,
            cursor_render_pipeline,
            cursor_uniform_buffer,
            cursor_position: Vec2::new(0.0, 0.0),
            cursor_visible: true,
        }
    }

    pub(super) fn update(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        let cursor_uniform = CursorUniform {
            position: [self.cursor_position.x, self.cursor_position.y],
            size: [18.0, 22.0],
            screen_size: [width as f32, height as f32],
            _padding: [0.0; 2],
        };

        queue.write_buffer(
            &self.cursor_uniform_buffer,
            0,
            bytemuck::bytes_of(&cursor_uniform),
        );
    }
}
