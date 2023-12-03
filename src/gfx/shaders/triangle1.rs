use wgpu::{SurfaceConfiguration, Device};

use super::shader_pipeline::{new_wgpu_shader_pipeline, draw_triangle_inputs_batched};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleVertexShaderInput {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl TriangleVertexShaderInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
}

impl TriangleVertexShaderInput {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TriangleVertexShaderInput>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct TriangleShaderPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffers: Vec<wgpu::Buffer>, // GPU memory
    vertex_inputs: Vec<[TriangleVertexShaderInput; 3]>, // CPU memory,
}

impl TriangleShaderPipeline {
    const NAME: &'static str = "triangle1";
    const BATCH_SIZE: usize = 100;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        Self {
            pipeline: new_wgpu_shader_pipeline(
                Self::NAME, 
                include_str!("triangle1.wgsl").into(), 
                &device, 
                &config, 
                TriangleVertexShaderInput::desc(),
                &[]),
            vertex_buffers: Vec::new(),
            vertex_inputs: Vec::new(),
        }
    }

    pub fn add_triangle(&mut self, input: &[TriangleVertexShaderInput; 3]) {
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