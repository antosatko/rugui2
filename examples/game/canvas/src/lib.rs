use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

pub struct Canvas<Color>
where
    Color: ColorRepr,
{
    pub pixels: Pixels<Color>,
    #[cfg(feature = "wgpu")]
    gpu_bound: Option<GpuBound>,
}

#[cfg(feature = "wgpu")]
struct GpuBound {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    dimensions_buffer: wgpu::Buffer,
    dimensions_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

#[derive(Debug, Clone)]
pub struct Pixels<Color>
where
    Color: ColorRepr,
{
    pub pixels: Vec<Color>,
    pub width: u32,
    pub height: u32,
}

impl<Color> Pixels<Color>
where
    Color: ColorRepr,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![Color::default(); (width * height) as usize],
            width,
            height,
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> &Color {
        &self.pixels[(y * self.width + x) as usize]
    }

    #[inline]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: Color) {
        self.pixels[(y * self.width + x) as usize] = color;
    }

    #[inline]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixels
            .resize((width * height) as usize, Color::default());
        self.width = width;
        self.height = height;
    }

    pub fn resize_retain(&mut self, width: u32, height: u32) {
        let mut new_pixels = vec![Color::default(); (width * height) as usize];
        for x in 0..width.min(self.width) {
            for y in 0..height.min(self.height) {
                new_pixels[(y * width + x) as usize] = *self.get_pixel(x, y);
            }
        }
        self.pixels = new_pixels;
        self.width = width;
        self.height = height;
    }

    pub fn into_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.pixels)
    }
}

/// new color representation trait
pub trait ColorRepr: Copy + Clone + Debug + Default + Pod + Zeroable + Sized {
    fn blend(&self, other: &Self) -> Self;
    fn alpha(&mut self, alpha: f32);

    fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    #[cfg(feature = "wgpu")]
    fn wgpu_format() -> wgpu::TextureFormat;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Rgba> for [f32; 4] {
    fn from(value: Rgba) -> Self {
        [
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            1.0-value.a as f32 / 255.0,
        ]
    }
}

impl ColorRepr for Rgba {
    fn blend(&self, other: &Self) -> Self {
        if other.a == 255 {
            return *other;
        } else if other.a == 0 {
            return *self;
        } else if self.a == 0 {
            return *other;
        }
        let (s_a, s_r, s_g, s_b) = (
            self.a as f32 / 255.0,
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        );
        let (o_a, o_r, o_g, o_b) = (
            other.a as f32 / 255.0,
            other.r as f32 / 255.0,
            other.g as f32 / 255.0,
            other.b as f32 / 255.0,
        );
        let a = s_a * (1.0 - o_a) + o_a;
        let r = (s_r * s_a * (1.0 - o_a) + o_r * o_a) / a;
        let g = (s_g * s_a * (1.0 - o_a) + o_g * o_a) / a;
        let b = (s_b * s_a * (1.0 - o_a) + o_b * o_a) / a;
        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: (a * 255.0) as u8,
        }
    }

    fn alpha(&mut self, alpha: f32) {
        self.a = (self.a as f32 * alpha).max(255.0).min(0.0) as u8;
    }

    #[cfg(feature = "wgpu")]
    fn wgpu_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    padding: u8,
}

impl ColorRepr for Rgb {
    fn blend(&self, other: &Self) -> Self {
        *other
    }

    fn alpha(&mut self, _alpha: f32) {}

    #[cfg(feature = "wgpu")]
    fn wgpu_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default, PartialEq)]
pub struct Depth {
    pub depth: f32,
}

impl ColorRepr for Depth {
    fn blend(&self, other: &Self) -> Self {
        *other
    }

    fn alpha(&mut self, _alpha: f32) {}

    #[cfg(feature = "wgpu")]
    fn wgpu_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Depth32Float
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default, PartialEq)]
/// A boolean value that can be used in a shader
///
/// internally represented as a u8
pub struct Bool(pub u8);

impl ColorRepr for Bool {
    fn blend(&self, other: &Self) -> Self {
        *other
    }

    fn alpha(&mut self, alpha: f32) {
        if alpha < 0.5 {
            *self = Bool::FALSE;
        } else {
            *self = Bool::TRUE;
        }
    }

    #[cfg(feature = "wgpu")]
    fn wgpu_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::R8Unorm
    }
}

impl Rgba {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const MAGENTA: Self = Self {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Rgb {
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        padding: u8::MAX,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        padding: u8::MAX,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        padding: u8::MAX,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        padding: u8::MAX,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        padding: u8::MAX,
    };
    pub const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
        padding: u8::MAX,
    };
    pub const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
        padding: u8::MAX,
    };
    pub const MAGENTA: Self = Self {
        r: 255,
        g: 0,
        b: 255,
        padding: u8::MAX,
    };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            padding: u8::MAX,
        }
    }
}

impl Depth {
    pub const ZERO: Self = Self { depth: 0.0 };
    pub const ONE: Self = Self { depth: 1.0 };

    pub fn new(depth: f32) -> Self {
        Self { depth }
    }
}

impl Bool {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(u8::MAX);

    pub fn new(value: bool) -> Self {
        Self(value as u8)
    }
}

impl<Color> Canvas<Color>
where
    Color: ColorRepr,
{
    #[cfg(feature = "wgpu")]
    pub const BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Canvas Bind Group Layout"),
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
            ],
        };

    #[cfg(feature = "wgpu")]
    pub const DIMENSIONS_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Dimensions Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };

    #[cfg(feature = "wgpu")]
    pub fn new_wgpu(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let pixels = Pixels::new(size.0, size.1);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Color::wgpu_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[Color::wgpu_format()],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas Buffer"),
            size: (size.0 * size.1 * Color::size() as u32) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dimensions Buffer"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let dimensions_bind_group_layout =
            device.create_bind_group_layout(&Self::DIMENSIONS_LAYOUT);

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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Canvas Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &dimensions_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("rgba.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Canvas Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
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

        let gpu_bound = Some(GpuBound {
            texture,
            view,
            sampler,
            buffer,
            bind_group,
            pipeline,
            dimensions_buffer,
            dimensions_bind_group,
        });

        Self { pixels, gpu_bound }
    }

    #[cfg(feature = "wgpu")]
    pub fn new(size: (u32, u32)) -> Self {
        let pixels = Pixels::new(size.0, size.1);
        Self {
            pixels,
            gpu_bound: None,
        }
    }

    #[cfg(not(feature = "wgpu"))]
    pub fn new(size: (u32, u32)) -> Self {
        let pixels = Pixels::new(size.0, size.1);
        Self { pixels }
    }

    #[cfg(feature = "wgpu")]
    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.pixels.resize_retain(size.0, size.1);

        let gpu_bound = match self.gpu_bound.as_mut() {
            Some(gpu_bound) => gpu_bound,
            None => return,
        };

        gpu_bound.texture.destroy();

        gpu_bound.texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Canvas Texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Color::wgpu_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[Color::wgpu_format()],
        });

        gpu_bound.view = gpu_bound
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        gpu_bound.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Canvas Buffer"),
            size: (size.0 * size.1 * Color::size() as u32) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        gpu_bound.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Canvas Bind Group"),
            layout: &device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu_bound.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gpu_bound.sampler),
                },
            ],
        });
    }

    #[cfg(not(feature = "wgpu"))]
    pub fn resize_retain(&mut self, size: (u32, u32)) {
        self.pixels.resize_retain(size.0, size.1);
    }

    #[cfg(feature = "wgpu")]
    pub fn render(
        &self,
        pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
    ) -> Result<(), wgpu::SurfaceError> {
        let gpu_bound = match self.gpu_bound.as_ref() {
            Some(gpu_bound) => gpu_bound,
            None => return Ok(()),
        };

        let size = self.pixels.dimensions();
        queue.write_buffer(
            &gpu_bound.dimensions_buffer,
            0,
            &bytemuck::cast_slice(&[size.0 as f32, size.1 as f32]),
        );
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &gpu_bound.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels.into_bytes(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(Color::size() as u32 * size.0),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );

        pass.set_pipeline(&gpu_bound.pipeline);
        pass.set_bind_group(0, &gpu_bound.bind_group, &[]);
        pass.set_bind_group(1, &gpu_bound.dimensions_bind_group, &[]);
        pass.draw(0..4, 0..1);

        Ok(())
    }

    #[cfg(feature = "wgpu")]
    pub fn blit_texture(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::TextureView,
    ) {
        let gpu = match &self.gpu_bound {
            Some(g) => g,
            None => return,
        };
        let blitter = wgpu::util::TextureBlitter::new(device, wgpu::TextureFormat::Rgba8UnormSrgb);
        blitter.copy(device, encoder, texture, &gpu.view);
    }

    #[inline]
    pub fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.pixels.width || y >= self.pixels.height {
            return;
        }
        self.pixels
            .put_pixel(x, y, self.pixels.get_pixel(x, y).blend(&color));
    }

    #[inline]
    pub fn blend_pixel_unchecked(&mut self, x: u32, y: u32, color: Color) {
        self.pixels
            .put_pixel(x, y, self.pixels.get_pixel(x, y).blend(&color));
    }

    pub fn clear(&mut self, color: Color) {
        const STRIDE: usize = 4;
        let len = self.pixels.pixels.len();
        let remainder = len % STRIDE + 1;
        for i in (STRIDE..len).step_by(STRIDE) {
            self.pixels.pixels[i] = color;
            self.pixels.pixels[i - 1] = color;
            self.pixels.pixels[i - 2] = color;
            self.pixels.pixels[i - 3] = color;
        }
        for i in 1..remainder {
            self.pixels.pixels[len - i] = color;
        }
    }
}

impl Canvas<Rgb> {
    pub fn blend_pixel_rgba(&mut self, x: u32, y: u32, color: Rgba) {
        if x >= self.pixels.width || y >= self.pixels.height {
            return;
        }
        let pixel = Rgba::from(*self.pixels.get_pixel(x, y));
        self.pixels.put_pixel(x, y, pixel.blend(&color).into());
    }

    pub fn blend_pixel_rgba_unchecked(&mut self, x: u32, y: u32, color: Rgba) {
        let pixel = Rgba::from(*self.pixels.get_pixel(x, y));
        self.pixels.put_pixel(x, y, pixel.blend(&color).into());
    }
}

pub enum Shapes {
    Rectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Circle {
        x: i32,
        y: i32,
        radius: i32,
    },
    Line {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },
    Point {
        x: i32,
        y: i32,
    },
}

impl<Color> Canvas<Color>
where
    Color: ColorRepr,
{
    pub fn draw_shape(&mut self, shape: Shapes, color: Color) {
        let bounds = Bounds {
            l_bound: 0,
            r_bound: self.pixels.width,
            t_bound: 0,
            b_bound: self.pixels.height,
        };
        match shape {
            Shapes::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                let Bounds {
                    l_bound,
                    r_bound,
                    t_bound,
                    b_bound,
                } = calc_bounds(
                    Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                    bounds,
                );
                for i in l_bound..r_bound {
                    for j in t_bound..b_bound {
                        self.blend_pixel_unchecked(i, j, color);
                    }
                }
            }
            Shapes::Circle { x, y, radius } => {
                let radius2 = radius * radius;
                let (l_bound, r_bound, t_bound, b_bound) =
                    (x - radius, x + radius, y - radius, y + radius);
                let Bounds {
                    l_bound,
                    r_bound,
                    t_bound,
                    b_bound,
                } = calc_bounds(
                    Rect {
                        x: l_bound as i32,
                        y: t_bound as i32,
                        width: (r_bound - l_bound) as i32,
                        height: (b_bound - t_bound) as i32,
                    },
                    bounds,
                );
                for i in l_bound..r_bound {
                    for j in t_bound..b_bound {
                        let dx = x as i32 - i as i32;
                        let dy = y as i32 - j as i32;
                        let dist = dx * dx + dy * dy;
                        if dist < radius2 as i32 {
                            self.blend_pixel_unchecked(i, j, color);
                        }
                    }
                }
            }
            Shapes::Line { x1, y1, x2, y2 } => {
                let dx = x2 as i32 - x1 as i32;
                let dy = y2 as i32 - y1 as i32;
                let dx2 = dx.abs() << 1;
                let dy2 = dy.abs() << 1;
                let sx = if dx >= 0 { 1 } else { -1 };
                let sy = if dy >= 0 { 1 } else { -1 };
                let mut x = x1 as i32;
                let mut y = y1 as i32;
                if dx2 >= dy2 {
                    let mut err = dy2 - dx2;
                    loop {
                        self.blend_pixel(x as u32, y as u32, color);
                        if x == x2 as i32 {
                            break;
                        }
                        if err > 0 {
                            y += sy;
                            err -= dx2;
                        }
                        x += sx;
                        err += dy2;
                    }
                } else {
                    let mut err = dx2 - dy2;
                    loop {
                        self.blend_pixel(x as u32, y as u32, color);
                        if y == y2 as i32 {
                            break;
                        }
                        if err > 0 {
                            x += sx;
                            err -= dy2;
                        }
                        y += sy;
                        err += dx2;
                    }
                }
            }
            Shapes::Point { x, y } => {
                if x < bounds.r_bound as i32
                    && y < bounds.b_bound as i32
                    && x >= bounds.l_bound as i32
                    && y >= bounds.t_bound as i32
                {
                    self.blend_pixel_unchecked(x as u32, y as u32, color);
                }
            }
        }
    }

    pub fn outline_shape(&mut self, shape: Shapes, outline: u32, color: Color) {
        let bounds = Bounds {
            l_bound: 0,
            r_bound: self.pixels.width,
            t_bound: 0,
            b_bound: self.pixels.height,
        };
        match shape {
            Shapes::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                let Bounds {
                    l_bound,
                    r_bound,
                    t_bound,
                    b_bound,
                } = calc_bounds(
                    Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                    bounds,
                );
                for i in l_bound..r_bound {
                    for j in t_bound..b_bound {
                        if i < l_bound + outline
                            || i >= r_bound - outline
                            || j < t_bound + outline
                            || j >= b_bound - outline
                        {
                            self.blend_pixel_unchecked(i, j, color);
                        }
                    }
                }
            }
            Shapes::Circle { x, y, radius } => {
                let radius2 = radius * radius;
                let (l_bound, r_bound, t_bound, b_bound) =
                    (x - radius, x + radius, y - radius, y + radius);
                let Bounds {
                    l_bound,
                    r_bound,
                    t_bound,
                    b_bound,
                } = calc_bounds(
                    Rect {
                        x: l_bound as i32,
                        y: t_bound as i32,
                        width: (r_bound - l_bound) as i32,
                        height: (b_bound - t_bound) as i32,
                    },
                    bounds,
                );
                for i in l_bound..r_bound {
                    for j in t_bound..b_bound {
                        let dx = x as i32 - i as i32;
                        let dy = y as i32 - j as i32;
                        let dist = dx * dx + dy * dy;
                        if dist < radius2 as i32 {
                            if dx.abs() < outline as i32 || dy.abs() < outline as i32 {
                                self.blend_pixel_unchecked(i, j, color);
                            }
                        }
                    }
                }
            }
            Shapes::Line { x1, y1, x2, y2 } => {
                self.draw_shape(Shapes::Line { x1, y1, x2, y2 }, color);
            }
            Shapes::Point { x, y } => {
                self.draw_shape(Shapes::Point { x, y }, color);
            }
        }
    }
}

struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub struct Bounds {
    pub l_bound: u32,
    pub r_bound: u32,
    pub t_bound: u32,
    pub b_bound: u32,
}

#[inline]
fn calc_bounds(shape: Rect, bounds: Bounds) -> Bounds {
    let l_bound = shape
        .x
        .max(bounds.t_bound as i32)
        .min(bounds.r_bound as i32) as u32;
    let r_bound = (shape.x + shape.width)
        .max(bounds.l_bound as i32)
        .min(bounds.r_bound as i32) as u32;
    let t_bound = shape
        .y
        .max(bounds.t_bound as i32)
        .min(bounds.b_bound as i32) as u32;
    let b_bound = (shape.y + shape.height)
        .max(bounds.t_bound as i32)
        .min(bounds.b_bound as i32) as u32;
    Bounds {
        l_bound,
        r_bound,
        t_bound,
        b_bound,
    }
}

impl From<Rgb> for Rgba {
    fn from(rgb: Rgb) -> Self {
        Self {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            a: 255,
        }
    }
}

impl From<Rgba> for Rgb {
    fn from(rgba: Rgba) -> Self {
        Self {
            r: rgba.r,
            g: rgba.g,
            b: rgba.b,
            padding: u8::MAX,
        }
    }
}

impl From<Bool> for Rgba {
    fn from(Bool(value): Bool) -> Self {
        if value == 0 {
            Rgba::BLACK
        } else {
            Rgba::WHITE
        }
    }
}

impl From<Rgba> for Bool {
    fn from(rgba: Rgba) -> Self {
        if rgba == Rgba::BLACK {
            Bool::FALSE
        } else {
            Bool::TRUE
        }
    }
}

impl From<Bool> for Rgb {
    fn from(Bool(value): Bool) -> Self {
        if value == 0 {
            Rgb::BLACK
        } else {
            Rgb::WHITE
        }
    }
}

impl From<Rgb> for Bool {
    fn from(rgb: Rgb) -> Self {
        if rgb == Rgb::BLACK {
            Bool::FALSE
        } else {
            Bool::TRUE
        }
    }
}

impl From<Bool> for Depth {
    fn from(Bool(value): Bool) -> Self {
        if value == 0 {
            Depth::ZERO
        } else {
            Depth::ONE
        }
    }
}

impl From<Depth> for Bool {
    fn from(depth: Depth) -> Self {
        if depth == Depth::ZERO {
            Bool::FALSE
        } else {
            Bool::TRUE
        }
    }
}

impl From<Rgb> for Depth {
    fn from(rgb: Rgb) -> Self {
        Depth::new(rgb.r as f32 / 255.0)
    }
}

impl From<Depth> for Rgb {
    fn from(depth: Depth) -> Self {
        Rgb::new((depth.depth * 255.0) as u8, 0, 0)
    }
}
