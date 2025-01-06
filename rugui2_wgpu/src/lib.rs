use std::{collections::HashMap, mem::size_of, num::NonZero};

use texture::{DepthBuffer, Texture};
use wgpu::{include_wgsl, PipelineLayoutDescriptor, RenderPipelineDescriptor, VertexAttribute};

use rugui2::element::{ElementInstance, ElementKey, Flags};

pub mod texture;

pub const BUFFER_SIZE: u64 = (1 << 20) / size_of::<WGPUElementInstance>() as u64;
pub const BUFFER_BYTES: u64 = BUFFER_SIZE * size_of::<WGPUElementInstance>() as u64;

pub struct Rugui2WGPU {
    pub dimensions_buffer: wgpu::Buffer,
    pub dimensions_bind_group: wgpu::BindGroup,
    pub depth_buffer: DepthBuffer,
    pub size: (u32, u32),

    instance_buffers: Vec<(wgpu::Buffer, Vec<WGPUElementInstance>)>,

    dummy_texture: Texture,

    pub pipeline: wgpu::RenderPipeline,
    pub stencil_pipeline: wgpu::RenderPipeline,
    pub end_stencil_pipeline: wgpu::RenderPipeline,
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
        self.prepare_buffers(gui.elements() as u64, device);
        gui.foreach_element_mut(
            &mut |e, k, _depth| {
                let (buffer, idx) = self.get_buffer_idx(k.raw());
                self.instance_buffers[buffer].1[idx as usize] =
                    WGPUElementInstance::from_instance(*e.instance());
            },
            None,
            0,
        );
        for (buffer, data) in &self.instance_buffers {
            match queue.write_buffer_with(buffer, 0, NonZero::new(BUFFER_BYTES).unwrap()) {
                Some(mut b) => {
                    b.copy_from_slice(bytemuck::cast_slice(data));
                }
                _ => (),
            }
        }
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
            ));
        }
    }

    pub fn get_buffer_idx(&self, i: u64) -> (usize, u64) {
        let buffer_idx = i / BUFFER_SIZE;
        let idx = i % BUFFER_SIZE;
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
}

#[derive(bytemuck::Zeroable, bytemuck::NoUninit, Debug, Copy, Clone, Default)]
#[repr(C)]
struct WGPUElementInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    pub flags: u32,
    pub round: [f32; 2],
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
            image_size: _,
            scroll: _,
        } = value;
        Self {
            pos: container.pos.into(),
            size: container.size.into(),
            rotation: container.rotation.into(),
            color,
            flags,
            round,
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
        }
    }
}
