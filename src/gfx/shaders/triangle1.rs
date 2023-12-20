use super::util::{new_wgpu_shader_pipeline, draw_triangle_inputs_batched};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleVertexShaderInput {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl TriangleVertexShaderInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

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
}

impl TriangleShaderPipeline {
    const NAME: &'static str = "triangle1";

    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            pipeline: new_wgpu_shader_pipeline(
                Self::NAME, 
                include_str!("triangle1.wgsl"), 
                device, 
                format, 
                &[TriangleVertexShaderInput::desc()],
                &[]),
        }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>, 
        queue: &wgpu::Queue, 
        vertex_buffer: &'a wgpu::Buffer,
        vertex_inputs: &'a [[TriangleVertexShaderInput; 3]],
    ) {
        draw_triangle_inputs_batched(
            vertex_inputs,
            3,
            vertex_buffer,
            &self.pipeline,
            render_pass,
            queue,
        );
    }
}