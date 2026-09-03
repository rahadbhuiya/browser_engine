use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};

pub mod image_decoder;
pub use image_decoder::{DecodedImage, ImageCache};

use crate::paint::{PaintCommand, Color};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub corner_radius: f32,
    pub _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewportUniform {
    pub size: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0] },
    Vertex { position: [1.0, 0.0] },
    Vertex { position: [0.0, 1.0] },
    Vertex { position: [1.0, 1.0] },
];

pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,

    // Pipeline & Buffers for Rectangles
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub viewport_buffer: wgpu::Buffer,
    pub viewport_bind_group: wgpu::BindGroup,

    // Text Rendering
    pub glyph_brush: GlyphBrush<()>,
    pub staging_belt: wgpu::util::StagingBelt,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // 1. Initialize WGPU Instance and Surface
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // The surface needs to live as long as the window. Since we store Window in Arc,
        // we can safely create a surface from it.
        let surface = instance.create_surface(window.clone()).unwrap();

        // 2. Request Adapter & Device
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // 3. Configure Surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // 4. Create Viewport Uniform Buffer and Bind Group
        let viewport_uniform = ViewportUniform {
            size: [size.width as f32, size.height as f32],
        };
        let viewport_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewport Uniform Buffer"),
            contents: bytemuck::cast_slice(&[viewport_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let viewport_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("viewport_bind_group_layout"),
            });

        let viewport_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &viewport_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_buffer.as_entire_binding(),
            }],
            label: Some("viewport_bind_group"),
        });

        // 5. Setup Shader & Pipeline for Rectangles
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&viewport_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    // Vertex buffer (Unit quad vertices)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    // Instance buffer (Quad positions, sizes, colors)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: 8,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // 6. Setup Text Rendering with wgpu_glyph
        // Arial font is standard on Windows. If it fails, fallback to standard bytes can be added,
        // but since we run on Lester's system (Windows), Fonts/arial.ttf will exist.
        let font_data = std::fs::read("C:\\Windows\\Fonts\\arial.ttf")
            .unwrap_or_else(|_| panic!("Failed to load arial.ttf font from system directories."));
        let font = ab_glyph::FontArc::try_from_vec(font_data).expect("Failed to parse arial.ttf");
        let glyph_brush = GlyphBrushBuilder::using_font(font).build(&device, config.format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        RenderState {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            viewport_buffer,
            viewport_bind_group,
            glyph_brush,
            staging_belt,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update viewport uniform
            let viewport_uniform = ViewportUniform {
                size: [new_size.width as f32, new_size.height as f32],
            };
            self.queue.write_buffer(
                &self.viewport_buffer,
                0,
                bytemuck::cast_slice(&[viewport_uniform]),
            );
        }
    }

    pub fn render(&mut self, paint_commands: &[PaintCommand], scroll_y: f32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // 1. Separate Rectangles and Text Commands
        let mut instances = Vec::new();
        let mut text_sections = Vec::new();

        for cmd in paint_commands {
            match cmd {
                PaintCommand::DrawRect { rect, color, border_radius, is_fixed } => {
                    let y_pos = if *is_fixed { rect.y } else { rect.y - scroll_y };
                    instances.push(InstanceRaw {
                        position: [rect.x, y_pos],
                        size: [rect.width, rect.height],
                        color: [color.r, color.g, color.b, color.a],
                        corner_radius: *border_radius,
                        _padding: [0.0; 3],
                    });
                }
                PaintCommand::DrawText { text, rect, color, font_size, is_fixed } => {
                    let y_pos = if *is_fixed { rect.y } else { rect.y - scroll_y };
                    text_sections.push((
                        text.clone(),
                        [rect.x, y_pos],
                        *color,
                        *font_size,
                    ));
                }
                PaintCommand::DrawImage { rect, alt, is_fixed, .. } => {
                    let y_pos = if *is_fixed { rect.y } else { rect.y - scroll_y };
                    instances.push(InstanceRaw {
                        position: [rect.x, y_pos],
                        size: [rect.width, rect.height],
                        color: [0.12, 0.16, 0.23, 1.0],
                        corner_radius: 6.0,
                        _padding: [0.0; 3],
                    });
                    let label = format!("🖼️ Image [{}]", if alt.is_empty() { "Graphic" } else { alt });
                    text_sections.push((
                        label,
                        [rect.x + 8.0, y_pos + 8.0],
                        Color::new(0.22, 0.74, 0.97, 1.0),
                        14.0,
                    ));
                }
            }
        }

        // 2. Upload Instance Buffer for Rectangles
        let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // 3. Render Pass for Rectangles
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.95,
                            g: 0.95,
                            b: 0.95,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !instances.is_empty() {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.viewport_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.draw(0..VERTICES.len() as u32, 0..instances.len() as u32);
            }
        }

        // 4. Queue Text to GlyphBrush
        for (text, pos, color, font_size) in text_sections {
            self.glyph_brush.queue(Section {
                screen_position: (pos[0], pos[1]),
                bounds: (self.size.width as f32 - pos[0], self.size.height as f32 - pos[1]),
                text: vec![Text::new(&text)
                    .with_color([color.r, color.g, color.b, color.a])
                    .with_scale(font_size)],
                ..Default::default()
            });
        }

        // 5. Render Pass for Text (drawn on top of rectangles)
        self.glyph_brush
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                self.size.width,
                self.size.height,
            )
            .expect("Failed to draw queued glyphs");

        // 6. Submit WGPU Commands
        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.staging_belt.recall();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_cache_decoding() {
        let png_bytes: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];

        let mut cache = ImageCache::new();
        assert!(cache.is_empty());
        let res = cache.load_from_memory("https://example.com/logo.png", &png_bytes);
        assert!(res.is_ok());
        let decoded = res.unwrap();
        assert_eq!(decoded.width, 1);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.rgba.len(), 4);
        assert_eq!(cache.len(), 1);
    }
}
