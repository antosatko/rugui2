use std::{num::NonZero, sync::Arc};

use canvas::Canvas;
use rugui2::Gui;
pub use rugui2_wgpu;
use rugui2_wgpu::{texture::Texture, Rugui2WGPU};
pub use rugui2_winit;

use crate::Msgs;

pub struct Drawing {
    pub config: wgpu::SurfaceConfiguration,
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub window: Arc<winit::window::Window>,
    pub size: (u32, u32),
    pub canvas: canvas::Canvas<canvas::Rgba>,
    pub game_tex: rugui2_wgpu::texture::Texture,
}

impl Drawing {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap();

        let canvas = Canvas::new_wgpu(&device, (1000, 800));

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface.get_capabilities(&adapter).formats.first().unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 3,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        let game_tex =
            rugui2_wgpu::texture::Texture::new(&device, (1000, 800), None).unwrap();

        surface.configure(&device, &config);

        Self {
            config,
            instance,
            surface,
            device: Arc::new(device),
            queue: Arc::new(queue),
            window,
            size: (size.width, size.height),
            game_tex,
            canvas,
        }
    }

    pub fn draw(&self, gui: &mut Gui<Msgs, Texture>, renderer: &mut Rugui2WGPU) {
        if self.size.0 == 0 || self.size.1 == 0 {
            return;
        }
        let size: (u32, u32) = self.window.inner_size().into();
        if size != self.size {
            return;
        }
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Game Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.game_tex.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                self.canvas.render(&mut pass, &self.queue).unwrap();
            }

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(renderer.get_depth_stencil_attachment()),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                renderer.render(gui, &mut pass);
            }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);
        self.window.pre_present_notify();
        output.present();
    }

    pub fn resize(&mut self, gui: &mut Gui<Msgs, Texture>, size: (u32, u32)) {
        self.config.width = size.0;
        self.config.height = size.1;
        self.size = (size.0, size.1);
        if size.0 == 0 || size.1 == 0 {
            return;
        }
        self.canvas.resize(&self.device, size);
        gui.resize((NonZero::new(size.0).unwrap(), NonZero::new(size.1).unwrap()));
        self.surface.configure(&self.device, &self.config);
    }
}
