use std::{collections::HashMap, mem::size_of, num::NonZero};

use etagere::{euclid::Size2D, Allocation, BucketedAtlasAllocator, Size};
use image::{DynamicImage, RgbaImage};
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    zeno::{Angle, Placement, Transform},
    FontRef, GlyphId,
};
use texture::{DepthBuffer, Texture};
use wgpu::{include_wgsl, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexAttribute};

use rugui2::{
    element::{ElementInstance, ElementKey, Flags},
    rich_text::{GlyphFlags, TextShape},
    text::{GlyphKey, Paragraph, PhysicalChar, TextProccesor},
};

pub mod texture;

pub const BUFFER_SIZE: u64 = (1 << 20) / size_of::<WGPUElementInstance>() as u64;
pub const BUFFER_BYTES: u64 = BUFFER_SIZE * size_of::<WGPUElementInstance>() as u64;
pub const GLYPH_ATLAS_SIDE: usize = 2048;
pub const GLYPH_ATLAS_DEPTH: usize = 3;
pub const GLYPH_BUFFER_SIZE: u64 = (1 << 20) / size_of::<WGPUGlyphInstance>() as u64;
pub const GLYPH_BUFFER_BYTES: u64 = GLYPH_BUFFER_SIZE * size_of::<WGPUGlyphInstance>() as u64;

pub struct Rugui2WGPU {
    pub dimensions_buffer: wgpu::Buffer,
    pub dimensions_bind_group: wgpu::BindGroup,
    pub depth_buffer: DepthBuffer,
    pub size: (u32, u32),

    instance_buffers: Vec<(wgpu::Buffer, Vec<WGPUElementInstance>, Vec<PerElementData>)>,

    pub dummy_texture: Texture,

    pub pipeline: wgpu::RenderPipeline,
    pub stencil_pipeline: wgpu::RenderPipeline,
    pub end_stencil_pipeline: wgpu::RenderPipeline,

    scaler_ctx: ScaleContext,
    scaler_image: Image,
    glyph_atlas_img: Vec<u8>,
    glyph_atlas_tex: Texture,
    glyph_pipeline: wgpu::RenderPipeline,
    glyph_atlas_allocators: Vec<BucketedAtlasAllocator>,
    glyph_atlas_map: HashMap<GlyphKey, (Allocation, Placement, u32)>,
    glyph_instance_buffers: Vec<(wgpu::Buffer, Vec<WGPUGlyphInstance>)>,
    glyph_instances: usize,
    last_written_glyph_atlas: u32,
    empty_glyph_key: (Allocation, Placement, u32),
    cursor_glyph_key: (Allocation, Placement, u32),
}

impl Rugui2WGPU {
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
        array_stride: size_of::<WGPUElementInstance>() as u64,
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
                format: wgpu::VertexFormat::Float32,
                shader_location: 5,
                offset: 40,
            },
            // shadow
            VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                shader_location: 6,
                offset: 44,
            },
            // alpha
            VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                shader_location: 7,
                offset: 48,
            },
            // lin_grad_p1+p2
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 8,
                offset: 52,
            },
            // lin_grad_p1_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 9,
                offset: 68,
            },
            // lin_grad_p2_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 10,
                offset: 84,
            },
            // rad_grad_p1+p2
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 11,
                offset: 100,
            },
            // rad_grad_p1_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 12,
                offset: 116,
            },
            // rad_grad_p2_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 13,
                offset: 132,
            },
            // image_tint
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 14,
                offset: 148,
            },
            // shadow_alpha
            VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                shader_location: 15,
                offset: 164,
            },
        ],
        step_mode: wgpu::VertexStepMode::Instance,
    };
    pub const GLYPH_VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> =
        wgpu::VertexBufferLayout {
            array_stride: size_of::<WGPUGlyphInstance>() as u64,
            attributes: &[
                // position
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                    offset: 0,
                },
                // offset
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                    offset: 8,
                },
                // size
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 2,
                    offset: 16,
                },
                // color
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 3,
                    offset: 24,
                },
                // uvd
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 4,
                    offset: 40,
                },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
        };

    pub fn new(queue: &wgpu::Queue, device: &wgpu::Device, size: (u32, u32)) -> Self {
        let dummy_texture =
            Texture::from_bytes(device, queue, &[0; 4], (1, 1), Some("Rugui2 dummy texture"))
                .unwrap();
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&Texture::BIND_GROUP_LAYOUT);
        let glyph_texture_bind_group_layout =
            device.create_bind_group_layout(&Texture::GLYPH_BIND_GROUP_LAYOUT);

        let depth_buffer = DepthBuffer::new(device, size);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rugui2 Pipeline Layout Descriptor"),
            bind_group_layouts: &[&dimensions_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shaders = device.create_shader_module(include_wgsl!("shaders/base.wgsl"));

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };

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
                        ("TEXTURE".to_string(), Flags::Image.into()),
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
                        ("TEXTURE".to_string(), Flags::Image.into()),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rugui2 Glyph Pipeline Layout Descriptor"),
            bind_group_layouts: &[
                &dimensions_bind_group_layout,
                &texture_bind_group_layout,
                &glyph_texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shaders = device.create_shader_module(include_wgsl!("shaders/glyph.wgsl"));

        let glyph_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Rugui2 Glyph Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_main"),
                module: &shaders,
                buffers: &[Self::GLYPH_VERTEX_BUFFER_LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &HashMap::from([(
                        String::from("GLYPH_ATLAS_SIDE"),
                        GLYPH_ATLAS_SIDE as f64,
                    )]),
                    ..Default::default()
                },
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: Some("fs_main"),
                module: &shaders,
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &HashMap::from([(
                        String::from("GLYPH_ATLAS_SIDE"),
                        GLYPH_ATLAS_SIDE as f64,
                    )]),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let stencil_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rugui2 Stencil Pipeline Layout Descriptor"),
            bind_group_layouts: &[&dimensions_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::IncrementClamp,
        };

        let stencil_shaders = device.create_shader_module(include_wgsl!("shaders/quad.wgsl"));

        let stencil_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Rugui2 Stencil Render Pipeline"),
            layout: Some(&stencil_pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_main"),
                module: &stencil_shaders,
                buffers: &[Self::VERTEX_BUFFER_LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: Some("fs_main"),
                module: &stencil_shaders,
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::empty(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::DecrementClamp,
        };

        let end_stencil_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Rugui2 Stencil Render Pipeline"),
            layout: Some(&stencil_pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_main"),
                module: &stencil_shaders,
                buffers: &[Self::VERTEX_BUFFER_LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: Some("fs_main"),
                module: &stencil_shaders,
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::empty(),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let scaler_ctx = ScaleContext::new();
        let scaler_image = Image::new();

        let mut glyph_atlas_allocators: Vec<BucketedAtlasAllocator> = (0..GLYPH_ATLAS_DEPTH)
            .map(|_| {
                BucketedAtlasAllocator::new(Size2D::new(
                    GLYPH_ATLAS_SIDE as i32,
                    GLYPH_ATLAS_SIDE as i32,
                ))
            })
            .collect();
        let glyph_atlas_map = HashMap::new();
        let glyph_instance_buffers = Vec::new();

        let empty = glyph_atlas_allocators[0]
            .allocate(Size2D::new(1, 1))
            .unwrap();
        let empty_glyph_key = (empty, Placement::default(), 0);

        let cursor = glyph_atlas_allocators[0]
            .allocate(Size2D::new(5, 5))
            .unwrap();
        let cursor_glyph_key = (
            cursor,
            Placement {
                width: 5,
                height: 5,
                ..Default::default()
            },
            0,
        );

        let mut glyph_atlas_img = vec![0; GLYPH_ATLAS_SIDE * GLYPH_ATLAS_SIDE * GLYPH_ATLAS_DEPTH];

        for x in cursor.rectangle.min.x as usize..cursor.rectangle.min.x as usize + 5 {
            for y in cursor.rectangle.min.y as usize..cursor.rectangle.min.y as usize + 5 {
                glyph_atlas_img[x + y * GLYPH_ATLAS_SIDE] = 255;
            }
        }
        let glyph_atlas_tex = Texture::atlas(device);

        Self {
            dimensions_buffer,
            dimensions_bind_group,
            depth_buffer,
            size,
            pipeline,
            stencil_pipeline,
            end_stencil_pipeline,
            dummy_texture,
            instance_buffers: Vec::new(),
            scaler_ctx,
            scaler_image,
            glyph_atlas_img,
            glyph_atlas_tex,
            glyph_pipeline,
            glyph_atlas_allocators,
            glyph_atlas_map,
            glyph_instance_buffers,
            glyph_instances: 0,
            last_written_glyph_atlas: 0,
            empty_glyph_key,
            cursor_glyph_key,
        }
    }

    fn try_allocate_glyph(&mut self, size: Size) -> Option<(Allocation, u32)> {
        for _ in 0..GLYPH_ATLAS_DEPTH {
            match self.glyph_atlas_allocators[self.last_written_glyph_atlas as usize].allocate(size)
            {
                Some(allocation) => return Some((allocation, self.last_written_glyph_atlas)),
                None => (),
            }
            self.last_written_glyph_atlas =
                (self.last_written_glyph_atlas + 1) % GLYPH_ATLAS_DEPTH as u32
        }
        None
    }

    pub fn get_depth_stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment {
        wgpu::RenderPassDepthStencilAttachment {
            depth_ops: None,
            stencil_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0),
                store: wgpu::StoreOp::Store,
            }),
            view: &self.depth_buffer.view,
        }
    }

    pub fn resize<Msg: Clone>(
        &mut self,
        gui: &mut rugui2::Gui<Msg, Texture>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        let size = gui.size();
        if self.size == size {
            return;
        }
        self.size = size;

        self.depth_buffer = DepthBuffer::new(device, size);
        queue.write_buffer(
            &self.dimensions_buffer,
            0,
            bytemuck::cast_slice(&[size.0 as f32, size.1 as f32]),
        );
    }

    pub fn prepare<Msg: Clone>(
        &mut self,
        gui: &mut rugui2::Gui<Msg, Texture>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        self.resize(gui, queue, device);
        self.prepare_buffers(gui.elements() as u64, device);
        self.glyph_instances = 0;
        if let Some(entry) = gui.get_entry() {
            self.prepare_element(entry, gui, device);
        }
        for (buffer, data, _) in &self.instance_buffers {
            match queue.write_buffer_with(buffer, 0, NonZero::new(BUFFER_BYTES).unwrap()) {
                Some(mut b) => {
                    b.copy_from_slice(bytemuck::cast_slice(data));
                }
                _ => (),
            }
        }
        for (buffer, data) in &self.glyph_instance_buffers {
            match queue.write_buffer_with(buffer, 0, NonZero::new(GLYPH_BUFFER_BYTES).unwrap()) {
                Some(mut b) => {
                    b.copy_from_slice(bytemuck::cast_slice(data));
                }
                _ => (),
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.glyph_atlas_tex.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &self.glyph_atlas_img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(GLYPH_ATLAS_SIDE as u32),
                rows_per_image: Some(GLYPH_ATLAS_SIDE as u32),
            },
            wgpu::Extent3d {
                width: GLYPH_ATLAS_SIDE as u32,
                height: GLYPH_ATLAS_SIDE as u32,
                depth_or_array_layers: GLYPH_ATLAS_DEPTH as u32,
            },
        );
    }

    fn prepare_element<Msg: Clone>(
        &mut self,
        key: ElementKey,
        gui: &mut rugui2::Gui<Msg, Texture>,
        device: &wgpu::Device,
    ) {
        let e = gui.get_element_unchecked(key);
        let elem_instance = e.instance();
        let cont = elem_instance.container.pos;
        let color = elem_instance.font_color;
        let (buffer, idx) = self.get_buffer_idx(key.raw());
        self.instance_buffers[buffer].1[idx as usize] =
            WGPUElementInstance::from_instance(*elem_instance);
        match e.styles().text.get() {
            Some(text) => {
                let mut w = 0.0;
                let mut top_plus_height = 0.0;
                let text_start = self.get_glyph_instance_index(self.glyph_instances as _);
                let physical_text = &text.text;
                // let mut line_idx = 0;
                for line in physical_text.lines.iter().take(physical_text.active_lines) {
                    let mut last_char_idx = line.start;
                    for wrap in line.wraps.iter().take(line.active_wraps) {
                        self.resize_to_add_glyphs(wrap.phys_chars.len(), device);
                        w = wrap.bb.left;
                        top_plus_height = wrap.bb.top + wrap.bb.height;
                        for char in wrap.phys_chars.iter().take(wrap.active_chars) {
                            let glyph_map_data =
                                match self.try_get_or_cache_glyph(&gui.text_ctx, *char) {
                                    Some(data) => data,
                                    None => continue,
                                };
                            let mut color = color;
                            if let Some(Some(selection)) = &text.variant.selection() {
                                if selection.sorted.0 <= char.idx && char.idx < selection.sorted.1 {
                                    color = [0.0, 0.0, 1.0, 1.0]
                                }
                            }
                            let instance = WGPUGlyphInstance {
                                uvd: [
                                    glyph_map_data.0.rectangle.min.x as f32
                                        / GLYPH_ATLAS_SIDE as f32,
                                    glyph_map_data.0.rectangle.min.y as f32
                                        / GLYPH_ATLAS_SIDE as f32,
                                    glyph_map_data.2 as f32 / (GLYPH_ATLAS_DEPTH as f32 - 1.0),
                                ],
                                color,
                                size: [
                                    glyph_map_data.1.width as f32,
                                    glyph_map_data.1.height as f32,
                                ],
                                position: [cont.0 + w, wrap.bb.top + wrap.bb.height + cont.1],
                                offset: [glyph_map_data.1.left as f32, glyph_map_data.1.top as f32],
                            };
                            let (buffer, idx) =
                                self.get_glyph_instance_index(self.glyph_instances as _);
                            self.glyph_instance_buffers[buffer].1[idx as usize] = instance;

                            self.glyph_instances += 1;
                            if let Some(editor) = &text.variant.editor() {
                                if editor.cursor.idx == char.idx
                                    && *gui.selection.current() == Some(key)
                                {
                                    let (buffer, idx) =
                                        self.get_glyph_instance_index(self.glyph_instances as _);
                                    let cursor = self.cursor_glyph_key;

                                    let instance = WGPUGlyphInstance {
                                        uvd: [
                                            cursor.0.rectangle.min.x as f32
                                                / GLYPH_ATLAS_SIDE as f32,
                                            cursor.0.rectangle.min.y as f32
                                                / GLYPH_ATLAS_SIDE as f32,
                                            0.0,
                                        ],
                                        color: [1.0, 1.0, 1.0, 1.0],
                                        size: [1.0, -elem_instance.font_size],
                                        position: [
                                            cont.0 + w,
                                            wrap.bb.top + wrap.bb.height + cont.1,
                                        ],
                                        offset: [cursor.1.left as f32, cursor.1.top as f32],
                                    };

                                    self.glyph_instance_buffers[buffer].1[idx as usize] = instance;
                                    self.glyph_instances += 1;
                                }
                            }
                            last_char_idx = char.idx;
                            w += char.width;
                        }
                    }
                    if let Some(editor) = &text.variant.editor() {
                        if editor.cursor.idx == last_char_idx + 1
                            && *gui.selection.current() == Some(key)
                        {
                            let (buffer, idx) =
                                self.get_glyph_instance_index(self.glyph_instances as _);
                            let cursor = self.cursor_glyph_key;

                            let instance = WGPUGlyphInstance {
                                uvd: [
                                    cursor.0.rectangle.min.x as f32 / GLYPH_ATLAS_SIDE as f32,
                                    cursor.0.rectangle.min.y as f32 / GLYPH_ATLAS_SIDE as f32,
                                    0.0,
                                ],
                                color: [1.0, 1.0, 1.0, 1.0],
                                size: [1.0, -elem_instance.font_size],
                                position: [cont.0 + w, top_plus_height + cont.1],
                                offset: [cursor.1.left as f32, cursor.1.top as f32],
                            };

                            self.glyph_instance_buffers[buffer].1[idx as usize] = instance;
                            self.glyph_instances += 1;
                        }
                    }
                    //line_idx += 1;
                }

                let text_end = self.get_glyph_instance_index(self.glyph_instances as _);
                let pi_data = &mut self.instance_buffers[buffer].2[idx as usize];
                pi_data.text = true;
                pi_data.text_start = text_start;
                pi_data.text_end = text_end;
            }
            _ => self.instance_buffers[buffer].2[idx as usize].text = false,
        }
        if let Some(children) = e.children.clone() {
            for i in 0..children.len() {
                self.prepare_element(children[i], gui, device);
            }
        }
    }

    fn try_get_or_cache_glyph(
        &mut self,
        ctx: &TextProccesor,
        char: PhysicalChar,
    ) -> Option<(Allocation, Placement, u32)> {
        match self.glyph_atlas_map.get(&char.glyph_key) {
            None => {
                let font_idx = char.glyph_key.font_idx;
                let font = ctx.get_font(font_idx);
                let size = (char.glyph_key.font_size as f32).max(1.0);
                
                self.raster_glyph(
                    &font,
                    size,
                    true,
                    char.glyph_key.glyph_id,
                    if (char.glyph_key.flags & GlyphFlags::Bold as u8) > 0 {
                        size * 0.025
                    } else {
                        0.0
                    },
                    if (char.glyph_key.flags & GlyphFlags::Italic as u8) > 0 {
                        20.0
                    } else {
                        0.0
                    },
                    0.0,
                    0.0,
                );
                let data;
                let placement = self.scaler_image.placement;
                if placement.width <= 0 || placement.height <= 0 {
                    self.glyph_atlas_map
                        .insert(char.glyph_key, self.empty_glyph_key);
                    data = self.empty_glyph_key;
                } else {
                    let allocator_size =
                        Size2D::new(placement.width as i32, placement.height as i32);
                    match self.try_allocate_glyph(allocator_size) {
                        Some((space, atlas_idx)) => {
                            let offset = GLYPH_ATLAS_SIDE * GLYPH_ATLAS_SIDE * atlas_idx as usize;
                            let mut i = 0;
                            for y in 0..placement.height {
                                for x in 0..placement.width {
                                    let alpha = self.scaler_image.data[i as usize];
                                    let (x, y) = (
                                        x + space.rectangle.min.x as u32,
                                        y + space.rectangle.min.y as u32,
                                    );
                                    let atlas_i = y * GLYPH_ATLAS_SIDE as u32 + x;
                                    self.glyph_atlas_img[atlas_i as usize + offset] = alpha;
                                    i += 1;
                                }
                            }
                            data = (space, placement, atlas_idx);
                            self.glyph_atlas_map.insert(char.glyph_key, data);
                        }
                        None => {
                            let mut img = DynamicImage::new_luma8(
                                GLYPH_ATLAS_SIDE as u32,
                                GLYPH_ATLAS_SIDE as u32 * GLYPH_ATLAS_DEPTH as u32,
                            )
                            .to_luma8();
                            img.clone_from_slice(
                                &self.glyph_atlas_img
                                    [0..GLYPH_ATLAS_SIDE * GLYPH_ATLAS_SIDE * GLYPH_ATLAS_DEPTH],
                            );

                            img.save("atlas.png").unwrap();
                            panic!("insufficent glyph atlas. For the love of god just fix it already pls\nGlyph atlas dumped into 'atlas.png'");
                            return None;
                        }
                    }
                }
                Some(data)
            }
            d => d.cloned(),
        }
    }

    pub fn experimental_text_rendering(&mut self, ctx: &TextProccesor, text: &TextShape) {
        let mut img = RgbaImage::new(text.bounds.width as u32, text.bounds.height as u32);

        for line in &text.lines {
            let mut w = line.bounds.left;

            for glyph in &line.chars {
                let (allocation, placement, layer) = match self.try_get_or_cache_glyph(ctx, *glyph)
                {
                    Some(g) => g,
                    None => continue,
                };
                let offset = (GLYPH_ATLAS_SIDE * GLYPH_ATLAS_SIDE * layer as usize) as u32;

                for x in 0..placement.width {
                    for y in 0..placement.height {
                        let atlas_i = (y + allocation.rectangle.min.y as u32) * GLYPH_ATLAS_SIDE as u32 + (x + offset + allocation.rectangle.min.x as u32);
                        let alpha = self.glyph_atlas_img[atlas_i as usize];
                        if alpha == 0 {
                            continue;
                        }

                        let (x, y) = (
                            (w.round() + x as f32).round() as i32 + placement.left,
                            y as i32 - placement.top + (line.height + line.bounds.top).round() as i32,
                        );
                        if let Some(pixel) = img.get_pixel_mut_checked(x as u32, y as u32) {
                            let color = line
                                .color
                                .map(|c| ((c * (alpha as f32 / 255.0)) * 255.0) as u8);
                            pixel.0 = color;
                        }
                    }
                }

                w += glyph.width;
            }
        }
        
        img.save("texthere.png").expect("I mean..");
    }

    fn prepare_buffers(&mut self, elements: u64, device: &wgpu::Device) {
        let len = elements / BUFFER_SIZE;
        for _ in self.instance_buffers.len() as u64..len + 1 {
            self.instance_buffers.push((
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Rugui2 Instance Buffer"),
                    size: (size_of::<WGPUElementInstance>() * BUFFER_SIZE as usize) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                vec![WGPUElementInstance::default(); BUFFER_SIZE as usize],
                vec![PerElementData::default(); BUFFER_SIZE as usize],
            ));
        }
    }

    fn resize_to_add_glyphs(&mut self, additional: usize, device: &wgpu::Device) {
        let fit_to = self.glyph_instances + additional;
        while self.glyph_instance_buffers.len() * (GLYPH_BUFFER_SIZE as usize) < fit_to {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Rugui2 Glyph Instance Buffer"),
                size: GLYPH_BUFFER_BYTES,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let cache = vec![WGPUGlyphInstance::default(); GLYPH_BUFFER_SIZE as usize];
            self.glyph_instance_buffers.push((buffer, cache));
        }
    }

    pub fn get_buffer_idx(&self, i: u64) -> (usize, u64) {
        let buffer_idx = i / BUFFER_SIZE;
        let idx = i % BUFFER_SIZE;
        (buffer_idx as usize, idx)
    }

    pub fn get_glyph_instance_index(&self, i: u64) -> (usize, u64) {
        let buffer_idx = i / GLYPH_BUFFER_SIZE;
        let idx = i % GLYPH_BUFFER_SIZE;
        (buffer_idx as usize, idx)
    }

    pub fn render<'a, Msg: Clone>(
        &'a mut self,
        gui: &mut rugui2::Gui<Msg, Texture>,
        pass: &mut wgpu::RenderPass<'a>,
    ) {
        let entry = if let Some(entry) = gui.get_entry() {
            entry
        } else {
            return;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.dimensions_bind_group, &[]);
        pass.set_bind_group(1, self.dummy_texture.bind_group.as_ref(), &[]);
        pass.set_bind_group(2, self.glyph_atlas_tex.bind_group.as_ref(), &[]);
        pass.set_vertex_buffer(0, self.instance_buffers[0].0.slice(..));

        self.render_element(gui, entry, pass, 0, &mut 0);
    }

    fn render_element<'a, Msg: Clone>(
        &mut self,
        gui: &mut rugui2::Gui<Msg, Texture>,
        key: ElementKey,
        pass: &mut wgpu::RenderPass<'a>,
        mut stencil_index: u32,
        instance_buffer: &mut usize,
    ) {
        let (buffer, i) = self.get_buffer_idx(key.raw());
        let i = i as u32;
        let prev_buffer_idx = *instance_buffer;
        let change_buffer = buffer != *instance_buffer;
        if change_buffer {
            pass.set_vertex_buffer(0, self.instance_buffers[buffer].0.slice(..));
            *instance_buffer = buffer;
        }
        let e = gui.get_element_mut_unchecked(key);
        let overflow_hidden = Flags::OverflowHidden.contained_in(e.instance().flags);

        if overflow_hidden {
            pass.set_pipeline(&self.stencil_pipeline);
            pass.set_stencil_reference(stencil_index);
            stencil_index += 1;
            pass.draw(0..6, i..i + 1);

            pass.set_stencil_reference(stencil_index);
            pass.set_pipeline(&self.pipeline);
        }
        if let Some(tex) = e.styles().image.get() {
            pass.set_bind_group(1, tex.data.bind_group.as_ref(), &[]);
        }

        pass.draw(0..6, i..i + 1);

        let pi_data = &self.instance_buffers[buffer].2[i as usize];
        if pi_data.text {
            pass.set_pipeline(&self.glyph_pipeline);
            pass.set_vertex_buffer(
                0,
                self.glyph_instance_buffers
                    .get(pi_data.text_start.0)
                    .expect(&format!("Font at: '{}' not loaded.", pi_data.text_start.0))
                    .0
                    .slice(..),
            );
            pass.draw(0..6, pi_data.text_start.1 as u32..pi_data.text_end.1 as u32);

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.instance_buffers[buffer].0.slice(..));
        }

        if let Some(children) = e.children.take() {
            for child in &children {
                self.render_element(gui, *child, pass, stencil_index, instance_buffer);
            }
            gui.get_element_mut_unchecked(key).children = Some(children);
        }

        if overflow_hidden {
            pass.set_pipeline(&self.end_stencil_pipeline);
            pass.set_stencil_reference(stencil_index);
            pass.draw(0..6, i..i + 1);

            pass.set_pipeline(&self.pipeline);
            pass.set_stencil_reference(stencil_index - 1);
        }

        if change_buffer {
            *instance_buffer = prev_buffer_idx;
            pass.set_vertex_buffer(0, self.instance_buffers[prev_buffer_idx].0.slice(..));
        }
    }

    fn raster_glyph(
        &mut self,
        font: &FontRef,
        size: f32,
        hint: bool,
        glyph_id: GlyphId,
        embolden: f32,
        skew: f32,
        x: f32,
        y: f32,
    ) -> bool {
        use swash::zeno::{Format, Vector};
        let mut scaler = self.scaler_ctx.builder(*font).size(size).hint(hint).build();

        scaler.scale_bitmap_into(glyph_id, StrikeWith::BestFit, &mut self.scaler_image);
        scaler.scale_color_bitmap_into(glyph_id, StrikeWith::BestFit, &mut self.scaler_image);

        let offset = Vector::new(x.fract(), y.fract());

        Render::new(&[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
            Source::Bitmap(StrikeWith::BestFit),
        ])
        .embolden(embolden)
        .transform(Some(Transform::skew(Angle::from_degrees(skew), Angle::ZERO)))
        .format(Format::Alpha)
        .offset(offset)
        .render_into(&mut scaler, glyph_id, &mut self.scaler_image)
    }
}

#[derive(bytemuck::Zeroable, bytemuck::NoUninit, Debug, Copy, Clone, Default, PartialEq)]
#[repr(C)]
struct WGPUElementInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    pub flags: u32,
    pub round: f32,
    pub shadow: f32,
    pub alpha: f32,
    /// x, y
    pub lin_grad_p1: [f32; 2],
    /// x, y
    pub lin_grad_p2: [f32; 2],
    pub lin_grad_color1: [f32; 4],
    pub lin_grad_color2: [f32; 4],
    /// x, y
    pub rad_grad_p1: [f32; 2],
    /// x, y
    pub rad_grad_p2: [f32; 2],
    pub rad_grad_color1: [f32; 4],
    pub rad_grad_color2: [f32; 4],
    pub image_tint: [f32; 4],
    pub shadow_alpha: f32,
}

impl WGPUElementInstance {
    fn from_instance(value: ElementInstance) -> Self {
        value.into()
    }
}

impl From<ElementInstance> for WGPUElementInstance {
    fn from(value: ElementInstance) -> Self {
        let ElementInstance {
            container,
            color,
            flags,
            round,
            alpha,
            lin_grad_p1,
            lin_grad_p2,
            lin_grad_color1,
            lin_grad_color2,
            rad_grad_p1,
            rad_grad_p2,
            rad_grad_color1,
            rad_grad_color2,
            image_tint,
            shadow,
            image_size: _,
            scroll: _,
            padding: _,
            shadow_alpha,
            font: _,
            font_size: _,
            font_color: _,
            text_wrap: _,
            text_align: _,
            margin: _,
        } = value;
        Self {
            pos: container.pos.into(),
            size: container.size.into(),
            rotation: container.rotation.into(),
            color,
            flags,
            round,
            shadow,
            alpha,
            lin_grad_p1: lin_grad_p1.into(),
            lin_grad_p2: lin_grad_p2.into(),
            lin_grad_color1,
            lin_grad_color2,
            rad_grad_p1: rad_grad_p1.into(),
            rad_grad_p2: rad_grad_p2.into(),
            rad_grad_color1,
            rad_grad_color2,
            image_tint,
            shadow_alpha,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct PerElementData {
    pub text: bool,
    pub text_start: (usize, u64),
    pub text_end: (usize, u64),
}

#[derive(bytemuck::Zeroable, bytemuck::NoUninit, Debug, Copy, Clone, Default, PartialEq)]
#[repr(C)]
struct WGPUGlyphInstance {
    pub position: [f32; 2],
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub uvd: [f32; 3],
}
