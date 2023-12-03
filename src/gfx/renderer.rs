use std::collections::{VecDeque, HashMap};

use crate::util::time::now_unix;

use super::{shaders::{triangle1::{TriangleVertexShaderInput, TriangleShaderPipeline}, text_texture1::{TextTextureShaderPipeline, TextTextureVertexShaderInput}}, text::font::{FontRasterizer, make_font_rasterizer}};

pub trait Renderer {
    fn resize(&mut self, width: u32, height: u32);
    fn get_fps(&self) -> u32;

    fn draw_tri_fan(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]);
    fn draw_tri_strip(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]);
    fn draw_text(&mut self, text: &str, color: ColorRGBA32f, x: f32, y: f32, font_size: f32, position: DrawTextPosition);
    fn present(&mut self, clear_color: ColorRGBA32f) -> Result<(), wgpu::SurfaceError>;
}

#[derive(Debug, Clone, Copy)]
pub enum DrawTextPosition {
    TopLeft
}

#[derive(Debug, Copy, Clone)]
pub struct ColorRGBA32f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRGBA32f {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        ColorRGBA32f { r, g, b, a}
    }
}


#[derive(Debug, Copy, Clone)]
pub struct ViewSpaceCoordinate {
    pub x: f32,
    pub y: f32,
}

impl ViewSpaceCoordinate {
    pub fn new(x: f32, y: f32) -> Self {
        ViewSpaceCoordinate { x, y }
    }
}

struct WgpuRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    frame_timestamps: VecDeque<f64>,

    triangle_shader_pipeline: TriangleShaderPipeline,

    text_shader_pipelines: HashMap<CachedTextPipelineK, CachedTextPipelineV>,
    font_rasterizer: Box<dyn FontRasterizer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CachedTextPipelineK {
    text: String,
    font_size: u32,
}

struct CachedTextPipelineV { 
    pipeline: TextTextureShaderPipeline,
    last_used: f64,
    width: u32,
    height: u32,
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn get_fps(&self) -> u32 {
        self.frame_timestamps.len() as u32
    }

    fn draw_tri_fan(&mut self, color: ColorRGBA32f, vertexes: &[ViewSpaceCoordinate]) {
        if vertexes.len() < 3 {
            panic!("draw_tri_fan() expected at least 3 vertexes, got {}. color={:?}, vertexes={:?}", 
                vertexes.len(),
                color, 
                vertexes);
        }
        for i in 2..vertexes.len() {
            self.triangle_shader_pipeline.add_triangle(&[
                self.tri_to_gpu( &color, &vertexes[0]),
                self.tri_to_gpu( &color, &vertexes[i-1]),
                self.tri_to_gpu( &color, &vertexes[i])]);
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
            self.triangle_shader_pipeline.add_triangle(&[
                self.tri_to_gpu( &color, &vertexes[i-2]),
                self.tri_to_gpu( &color, &vertexes[i-1]),
                self.tri_to_gpu( &color, &vertexes[i])]);
        }
    }

    fn draw_text(&mut self, text: &str, color: ColorRGBA32f, x: f32, y: f32, font_size: f32, position: DrawTextPosition) {
        let font_size = font_size as u32;
        let k = CachedTextPipelineK { text: text.to_owned(), font_size };

        if !self.text_shader_pipelines.contains_key(&k) {
            let bytes_2d = self.font_rasterizer.rasterize_text_line(text, font_size as f32);
            if bytes_2d.len() == 0 {
                return;
            }
            let width = bytes_2d[0].len() as u32;
            let height = bytes_2d.len() as u32;
            let bytes_1d: Vec<u8> = bytes_2d.into_iter().flatten().collect();
            let pipeline = TextTextureShaderPipeline::new(&self.queue, &self.device, &self.config, bytes_1d.as_slice(), width, height);
            let v = CachedTextPipelineV {
                pipeline,
                last_used: 0.0, //dummy
                width,
                height,
            };
            self.text_shader_pipelines.insert(k.clone(), v);
        } 

        let width: f32;
        let height: f32;
        {
            let cached_v = self.text_shader_pipelines.get_mut(&k).expect("unable to get cached text pipeline (1)");
            cached_v.last_used = now_unix();
            width = cached_v.width as f32;
            height = cached_v.height as f32;
        }
        
        let (x, y) = match position {
            DrawTextPosition::TopLeft => (x, y),
        };

        let vertexes = &[
            self.text_tri_to_gpu(&color, &ViewSpaceCoordinate::new(x, y), [0.0, 0.0]),
            self.text_tri_to_gpu(&color, &ViewSpaceCoordinate::new(x + width, y), [1.0, 0.0]),
            self.text_tri_to_gpu(&color, &ViewSpaceCoordinate::new(x + width, y + height), [1.0, 1.0]),
            self.text_tri_to_gpu(&color, &ViewSpaceCoordinate::new(x, y + height), [0.0, 1.0]),
        ];

        let cached_v = self.text_shader_pipelines.get_mut(&k).expect("unable to get cached text pipeline (2)");
        cached_v.pipeline.add_triangle(&[vertexes[0], vertexes[1], vertexes[2]]);
        cached_v.pipeline.add_triangle(&[vertexes[2], vertexes[3], vertexes[0]]);
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

            self.triangle_shader_pipeline.draw(&mut render_pass, &self.device, &self.queue);
            self.text_shader_pipelines.values_mut().for_each(|x| x.pipeline.draw(&mut render_pass, &self.device, &self.queue));
        }
        let now = now_unix();
        self.text_shader_pipelines.retain(|_, v| now - v.last_used < 1.0);

        self.queue.submit(std::iter::once(encoder.finish()));

        let now = now_unix();
        self.frame_timestamps.push_back(now);
        while !self.frame_timestamps.is_empty() && *self.frame_timestamps.front().unwrap() < now - 1.0 {
            self.frame_timestamps.pop_front();
        }

        output.present();

        Ok(())
    }
}

impl WgpuRenderer {
    fn x_to_ndc(&self, x: f32) -> f32 {
        2.0 * x / (self.config.width as f32) - 1.0
    }

    fn y_to_ndc(&self, y: f32) -> f32 {
        1.0 - 2.0 * y / (self.config.height as f32)
    }

    fn tri_to_gpu(&self, color: &ColorRGBA32f, v: &ViewSpaceCoordinate) -> TriangleVertexShaderInput {
        TriangleVertexShaderInput{
            position: [self.x_to_ndc(v.x), self.y_to_ndc(v.y)], 
            color: [color.r, color.g, color.b, color.a],
        } 
    }

    fn text_tri_to_gpu(&self, color: &ColorRGBA32f, v: &ViewSpaceCoordinate, tex_coords: [f32; 2]) -> TextTextureVertexShaderInput {
        TextTextureVertexShaderInput{
            position: [self.x_to_ndc(v.x), self.y_to_ndc(v.y)], 
            color: [color.r, color.g, color.b, color.a],
            tex_coords,
        } 
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

    let triangle_shader_pipeline = TriangleShaderPipeline::new(&device, &config);

    let renderer = WgpuRenderer {
        surface,
        device,
        queue,
        config,
        frame_timestamps: VecDeque::new(),
        triangle_shader_pipeline,
        text_shader_pipelines: HashMap::new(),
        font_rasterizer: make_font_rasterizer(),
    };

    Box::new(renderer)
}