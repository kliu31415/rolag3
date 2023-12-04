use std::borrow::Cow;

use bytemuck::Pod;
use wgpu::{SurfaceConfiguration, Device, COPY_BUFFER_ALIGNMENT};

pub fn new_wgpu_shader_pipeline(
    name: &str,
    shader_code: &str, 
    device: &Device, 
    config: &SurfaceConfiguration, 
    vertex_buffer_layout: wgpu::VertexBufferLayout,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("{} Shader", name)),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_code)),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{} Render Pipeline Layout ", name)),
        bind_group_layouts: bind_group_layouts,
        push_constant_ranges: &[],
    });
    
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{} Render Pipeline", name)),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                vertex_buffer_layout,
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
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn draw_triangle_inputs_batched<'a, T: Pod>(
    vertex_inputs: Vec<T>, 
    vertexes_per_input: usize,
    vertex_buffer: &'a wgpu::Buffer, 
    pipeline: &'a wgpu::RenderPipeline,
    render_pass: &mut wgpu::RenderPass<'a>, 
    queue: &wgpu::Queue,
    bind_groups: &'a [wgpu::BindGroup],
) {
    render_pass.set_pipeline(&pipeline);
    for (i, bind_group) in bind_groups.iter().enumerate() {
        render_pass.set_bind_group(i as u32, bind_group, &[]);
    }
    let bytes: &[u8] = bytemuck::cast_slice(&vertex_inputs);
    if bytes.len() % (COPY_BUFFER_ALIGNMENT as usize) != 0 {
        todo!("wgpu copy buffer alignment isn't respected. Buffer size={}, desired alignment={}", 
            bytes.len(), 
            COPY_BUFFER_ALIGNMENT);
    }
    queue.write_buffer(vertex_buffer, 0u64, bytes);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(0..(bytes.len() as u64)));
    render_pass.draw(0..((vertex_inputs.len() * vertexes_per_input) as u32), 0..1);
}