use std::collections::HashMap;

use wgpu::{include_wgsl, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexAttribute};

use crate::ElementInstance;

#[repr(u8)]
pub enum Flags {
    LinearGradient = 0,
    RadialGradient,
    Count,
}

impl From<Flags> for f64 {
    fn from(value: Flags) -> Self {
        (1 << value as u64) as f64
    }
}

impl Flags {
    pub const NONE: u64 = 0;
}

pub struct Drawing {
    pub dimensions_buffer: wgpu::Buffer,
    pub dimensions_bind_group: wgpu::BindGroup,
    pub size: (u32, u32),

    pub instance_buffer: wgpu::Buffer,

    pub pipeline: wgpu::RenderPipeline,
}

impl Drawing {
    pub const DIMENSIONS_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Dimensions Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    pub const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<ElementInstance>() as u64,
        attributes: &[
            // center
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 0,
                offset: 0,
            },
            // size
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 1,
                offset: 8,
            },
            // rotation
            VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                shader_location: 2,
                offset: 16,
            },
            // color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 3,
                offset: 20,
            },
            // flags
            VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                shader_location: 4,
                offset: 36,
            },
            // round
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 5,
                offset: 40,
            },
            // alpha
            VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                shader_location: 6,
                offset: 48,
            },
        ],
        step_mode: wgpu::VertexStepMode::Instance,
    };

    pub fn new(queue: &wgpu::Queue, device: &wgpu::Device, size: (u32, u32)) -> Self {
        let dimensions_bind_group_layout =
            device.create_bind_group_layout(&Self::DIMENSIONS_LAYOUT);

        let dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dimensions Buffer"),
            size: std::mem::size_of::<(u32, u32)>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let dimensions_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dimensions Bind Group"),
            layout: &dimensions_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &dimensions_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        queue.write_buffer(
            &dimensions_buffer,
            0,
            bytemuck::cast_slice(&[size.0 as f32, size.1 as f32]),
        );

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rugui2 Instance Buffer"),
            size: (size_of::<ElementInstance>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rugui2 Pipeline Layout Descriptor"),
            bind_group_layouts: &[&dimensions_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shaders = device.create_shader_module(include_wgsl!("shaders/base.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Rugui2 Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_main"),
                module: &shaders,
                buffers: &[Self::VERTEX_BUFFER_LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &HashMap::from([
                        ("LIN_GRADIENT".to_string(), Flags::LinearGradient.into()),
                        ("RAD_GRADIENT".to_string(), Flags::RadialGradient.into()),
                    ]),
                    ..Default::default()
                },
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: Some("fs_main"),
                module: &shaders,
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &HashMap::from([
                        ("LIN_GRADIENT".to_string(), Flags::LinearGradient.into()),
                        ("RAD_GRADIENT".to_string(), Flags::RadialGradient.into()),
                    ]),
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            dimensions_buffer,
            dimensions_bind_group,
            size,
            pipeline,
            instance_buffer,
        }
    }

    pub fn resize(&mut self, size: (u32, u32), queue: &wgpu::Queue) {
        if self.size == size {
            return;
        }
        self.size = size;

        queue.write_buffer(
            &self.dimensions_buffer,
            0,
            bytemuck::cast_slice(&[size.0 as f32, size.1 as f32]),
        );
    }
}
