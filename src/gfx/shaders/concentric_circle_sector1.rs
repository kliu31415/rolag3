use wgpu::{SurfaceConfiguration, Device};

use super::shader_pipeline::{new_wgpu_shader_pipeline, draw_triangle_inputs_batched};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ConcrenticCircleSectorVertexShaderInput {
    pub position: [f32; 2],
    pub pixel_xy: [f32; 2],
    pub center: [f32; 2],
    pub r1: f32,
    pub r2: f32,
    pub color1: [f32; 4],
    pub color2: [f32; 4],
    pub theta_range1: [f32; 2],
    pub theta_range2: [f32; 2],
}

impl ConcrenticCircleSectorVertexShaderInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 9] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32,
        4 => Float32,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x2,
        8 => Float32x2,
    ];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ConcrenticCircleSectorVertexShaderInput>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct ConcrenticCircleSectorShaderPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffers: Vec<wgpu::Buffer>, // GPU memory
    vertex_inputs: Vec<[ConcrenticCircleSectorVertexShaderInput; 3]>, // CPU memory,
}

impl ConcrenticCircleSectorShaderPipeline {
    const NAME: &'static str = "concrentic_circle_sector1";
    const BATCH_SIZE: usize = 100;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        Self {
            pipeline: new_wgpu_shader_pipeline(
                Self::NAME, 
                include_str!("concentric_circle_sector1.wgsl").into(), 
                &device, 
                &config, 
                ConcrenticCircleSectorVertexShaderInput::desc(),
                &[]),
            vertex_buffers: Vec::new(),
            vertex_inputs: Vec::new(),
        }
    }

    pub fn add_triangle(&mut self, input: &[ConcrenticCircleSectorVertexShaderInput; 3]) {
        self.vertex_inputs.push(*input);
    }

    pub fn draw<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, device: &wgpu::Device, queue: &wgpu::Queue) {
        draw_triangle_inputs_batched(
            Self::NAME,
            Self::BATCH_SIZE,
            &mut self.vertex_inputs,
            3,
            &mut self.vertex_buffers,
            &mut self.pipeline,
            render_pass,
            device,
            queue,
            &[],
        );
    }
}