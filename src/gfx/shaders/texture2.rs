use crate::gfx::renderer::TextureAndMetadata;

use super::util::{new_wgpu_shader_pipeline, draw_triangle_inputs_batched_bg1};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Texture2VertexShaderInput {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color_mod: [f32; 4],
}

impl Texture2VertexShaderInput {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];
    
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Texture2VertexShaderInput>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// this shader pipeline takes in a 2d pixel array of bytes. Each byte represents the alpha value.
pub struct Texture2ShaderPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Texture2ShaderPipeline {
    const NAME: &'static str = "Texture2";

    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let pipeline = new_wgpu_shader_pipeline(
            Self::NAME, 
            include_str!("texture2.wgsl"), 
            device, 
            format, 
            &[Texture2VertexShaderInput::desc()],
            &[&TextureAndMetadata::get_standard_bind_group_layout(device)]);

        Self {
            pipeline,
        }
    }
    
    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>, 
        queue: &wgpu::Queue,
        vertex_buffer: &'a wgpu::Buffer,
        vertex_inputs: &'a [[Texture2VertexShaderInput; 3]],
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

