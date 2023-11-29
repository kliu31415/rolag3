use wgpu::{Device, SurfaceConfiguration, BufferDescriptor, COPY_BUFFER_ALIGNMENT};

pub trait Renderer {
    fn resize(&mut self, width: u32, height: u32);

    fn draw_tri_fan(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]);
    fn draw_tri_strip(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]);
    fn present(&mut self, clear_color: ColorRGBA32f) -> Result<(), wgpu::SurfaceError>;
}

#[derive(Debug, Copy, Clone)]
pub struct ColorRGBA32f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}


#[derive(Debug, Copy, Clone)]
pub struct ViewSpaceCoordinate {
    pub x: f32,
    pub y: f32,
}

struct ColoredVertex {
    color: ColorRGBA32f,
    coordinate: ViewSpaceCoordinate,
}
struct WgpuRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    triangle_pipeline: wgpu::RenderPipeline,
    triangle_vertex_buffers: Vec<wgpu::Buffer>, //GPU memory
    triangle_vertexes: Vec<ColoredVertex>, //CPU memory
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn draw_tri_fan(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]) {
        if vertexes.len() < 3 {
            panic!("draw_tri_fan() expected at least 3 vertexes, got {}. color={:?}, vertexes={:?}", 
                vertexes.len(),
                color, 
                vertexes);
        }
        for i in 2..vertexes.len() {
            self.triangle_vertexes.push(ColoredVertex { color, coordinate: vertexes[0] });
            self.triangle_vertexes.push(ColoredVertex { color, coordinate: vertexes[i-1] });
            self.triangle_vertexes.push(ColoredVertex { color, coordinate: vertexes[i] });
        }
    }

    fn draw_tri_strip(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]) {
        if vertexes.len() < 3 {
            panic!("draw_tri_strip() expected at least 3 vertexes, got {}. color={:?}, vertexes={:?}", 
                vertexes.len(), 
                color, 
                vertexes);
        }
        for i in 2..vertexes.len() {
            self.triangle_vertexes.push(ColoredVertex { color: color, coordinate: vertexes[i-2] });
            self.triangle_vertexes.push(ColoredVertex { color: color, coordinate: vertexes[i-1] });
            self.triangle_vertexes.push(ColoredVertex { color: color, coordinate: vertexes[i] });
        }
    }

    fn present(&mut self, clear_color: ColorRGBA32f) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: clear_color.r as f64,
                                g: clear_color.g as f64,
                                b: clear_color.b as f64,
                                a: clear_color.a as f64,
                            }
                        ),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            //render triangles
            self.draw_triangles(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.triangle_vertexes.clear();

        Ok(())
    }
}

impl WgpuRenderer {
    const TRIANGLE_BATCH_SIZE: usize = 100;

    fn draw_triangles<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        let ndc_vertexes = self.triangle_vertexes
            .iter()
            .map(|v| 
                TriangleVertexShaderInput{
                    position: [self.x_to_ndc(v.coordinate.x), self.y_to_ndc(v.coordinate.y)], 
                    color: [v.color.r, v.color.g, v.color.b, v.color.a],
                } )
            .collect::<Vec<_>>();
        while self.triangle_vertex_buffers.len() < ndc_vertexes.chunks(3 * Self::TRIANGLE_BATCH_SIZE).len() {
            let buffer = self.device.create_buffer(
                &BufferDescriptor { 
                    label: Some(&format!("Triangle Vertex Buffer #{}", self.triangle_vertex_buffers.len())), 
                    size: (Self::TRIANGLE_BATCH_SIZE * 3 * std::mem::size_of::<TriangleVertexShaderInput>()) as u64, 
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
                    mapped_at_creation: false,
                },
            );
            self.triangle_vertex_buffers.push(buffer);
        }

        render_pass.set_pipeline(&self.triangle_pipeline);
        for (i, batch) in ndc_vertexes.chunks(3 * Self::TRIANGLE_BATCH_SIZE).enumerate() {
            let bytes: &[u8] = bytemuck::cast_slice(batch);
            if bytes.len() % (COPY_BUFFER_ALIGNMENT as usize) != 0 {
                todo!("wgpu copy buffer alignment isn't respected. Buffer size={}, desired alignment={}", 
                    bytes.len(), 
                    COPY_BUFFER_ALIGNMENT);
            }
            self.queue.write_buffer(&self.triangle_vertex_buffers[i], 0u64, bytes);
            render_pass.set_vertex_buffer(0, self.triangle_vertex_buffers[i].slice(0..(bytes.len() as u64)));
            render_pass.draw(0..(batch.len() as u32), 0..1);
        }
    }

    fn x_to_ndc(&self, x: f32) -> f32 {
        2.0 * x / (self.config.width as f32) - 1.0
    }

    fn y_to_ndc(&self, y: f32) -> f32 {
        1.0 - 2.0 * y / (self.config.height as f32)
    }

}

pub fn make_renderer(window: &winit::window::Window) -> Box<dyn Renderer> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe {instance.create_surface(&window)}.unwrap();

    let adapter = pollster::block_on(
        instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        )
    ).unwrap();

    let (device, queue) = pollster::block_on(
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
        None,
        )
    ).unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let triangle_pipeline = make_triangle_pipeline(&device, &config);

    let renderer = WgpuRenderer {
        surface,
        device,
        queue,
        config,
        triangle_pipeline,
        triangle_vertex_buffers: Vec::new(),
        triangle_vertexes: Vec::new(),
    };

    Box::new(renderer)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TriangleVertexShaderInput {
    position: [f32; 2],
    color: [f32; 4],
}

impl TriangleVertexShaderInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TriangleVertexShaderInput>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

fn make_triangle_pipeline(device: &Device, config: &SurfaceConfiguration) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/triangle1.wgsl"));

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Triangle Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Triangle Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                TriangleVertexShaderInput::desc(),
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })]
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
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
            mask: !0, alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}