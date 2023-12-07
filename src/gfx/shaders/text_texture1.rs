use wgpu::SurfaceConfiguration;

use super::shader_pipeline::{new_wgpu_shader_pipeline, draw_triangle_inputs_batched_bg1};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextTextureVertexShaderInput {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl TextTextureVertexShaderInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4, 2 => Float32x2];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextTextureVertexShaderInput>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// this shader pipeline takes in a 2d pixel array of bytes. Each byte represents the alpha value.
pub struct TextTextureShaderPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl TextTextureShaderPipeline {
    const NAME: &'static str = "TextTexture1";

    pub fn new(device: &wgpu::Device, config: &SurfaceConfiguration, bgl: &[&wgpu::BindGroupLayout]) -> Self {
        let pipeline = new_wgpu_shader_pipeline(
            Self::NAME, 
            include_str!("text_texture1.wgsl"), 
            device, 
            config, 
            TextTextureVertexShaderInput::desc(),
            bgl);

        Self {
            pipeline,
        }
    }
    
    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>, 
        queue: &wgpu::Queue,
        vertex_buffer: &'a wgpu::Buffer,
        vertex_inputs: Vec<[TextTextureVertexShaderInput; 3]>,
        bg: &'a [wgpu::BindGroup],
    ) {
        draw_triangle_inputs_batched_bg1(
            vertex_inputs,
            3,
            vertex_buffer,
            &self.pipeline,
            render_pass,
            queue,
            bg,
        );
    }
}

