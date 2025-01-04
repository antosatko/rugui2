use std::{collections::HashMap, mem::size_of};

use texture::{DepthBuffer, Texture};
use wgpu::{include_wgsl, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexAttribute};

use rugui2::element::{ElementInstance, ElementKey, Flags};

pub mod texture;

pub struct Rugui2WGPU {
    pub dimensions_buffer: wgpu::Buffer,
    pub dimensions_bind_group: wgpu::BindGroup,
    pub depth_buffer: DepthBuffer,
    pub size: (u32, u32),

    pub instance_buffer: wgpu::Buffer,

    dummy_texture: Texture,

    pub pipeline: wgpu::RenderPipeline,
    pub stencil_pipeline: wgpu::RenderPipeline,
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
            // lin_grad_p1+p2
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 7,
                offset: 52,
            },
            // lin_grad_p1_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 8,
                offset: 68,
            },
            // lin_grad_p2_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 9,
                offset: 84,
            },
            // rad_grad_p1+p2
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 10,
                offset: 100,
            },
            // rad_grad_p1_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 11,
                offset: 116,
            },
            // rad_grad_p2_color
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 12,
                offset: 132,
            },
            // image_tint
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 13,
                offset: 148,
            },
            // image_size
            VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                shader_location: 14,
                offset: 164,
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

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rugui2 Instance Buffer"),
            size: (size_of::<ElementInstance>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let depth_buffer = DepthBuffer::new(device, size);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rugui2 Pipeline Layout Descriptor"),
            bind_group_layouts: &[&dimensions_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shaders = device.create_shader_module(include_wgsl!("shaders/base.wgsl"));

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::LessEqual,
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

        let stencil_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rugui2 Stencil Pipeline Layout Descriptor"),
            bind_group_layouts: &[&dimensions_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Replace,
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

        Self {
            dimensions_buffer,
            dimensions_bind_group,
            depth_buffer,
            size,
            pipeline,
            stencil_pipeline,
            dummy_texture,
            instance_buffer,
        }
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
        gui.foreach_element_mut(
            &mut |e, k, _depth| {
                queue.write_buffer(
                    &self.instance_buffer,
                    k.raw() * size_of::<ElementInstance>() as u64,
                    bytemuck::cast_slice(&[*e.instance()]),
                );
            },
            None,
            0,
        );
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
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

        self.render_element(gui, entry, pass, 0);
    }

    fn render_element<'a, Msg: Clone>(
        &mut self,
        gui: &mut rugui2::Gui<Msg, Texture>,
        key: ElementKey,
        pass: &mut wgpu::RenderPass<'a>,
        mut stencil_index: u32,
    ) {
        let i = key.raw() as u32;
        let e = gui.get_element_mut_unchecked(key);
        let overflow_hidden = Flags::OverflowHidden.contained_in(e.instance().flags);

        if overflow_hidden {
            pass.set_pipeline(&self.stencil_pipeline);
            stencil_index += 1;
            pass.set_stencil_reference(stencil_index);
            pass.draw(0..6, i..i + 1);

            pass.set_pipeline(&self.pipeline);
        }

        if let Some(tex) = e.styles().image.get() {
            pass.set_bind_group(1, tex.data.bind_group.as_ref(), &[]);
        }

        pass.draw(0..6, i..i + 1);

        if let Some(children) = e.children.take() {
            for child in &children {
                self.render_element(gui, *child, pass, stencil_index);
            }
            gui.get_element_mut_unchecked(key).children = Some(children);
        }

        if overflow_hidden {
            pass.set_pipeline(&self.stencil_pipeline);
            stencil_index -= 1;
            pass.set_stencil_reference(stencil_index);
            pass.draw(0..6, i..i + 1);

            pass.set_pipeline(&self.pipeline);
        }
    }
}
