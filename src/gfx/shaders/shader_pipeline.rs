use std::borrow::Cow;

use bytemuck::Pod;
use wgpu::{SurfaceConfiguration, Device, COPY_BUFFER_ALIGNMENT, BufferDescriptor};

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
    name: &str,
    batch_size: usize,
    vertex_inputs: &'a mut Vec<T>, 
    vertexes_per_input: usize,
    vertex_buffers: &'a mut Vec<wgpu::Buffer>, 
    pipeline: &'a wgpu::RenderPipeline,
    render_pass: &mut wgpu::RenderPass<'a>, 
    device: &wgpu::Device, 
    queue: &wgpu::Queue,
    bind_groups: &'a [wgpu::BindGroup],
) {
    while vertex_buffers.len() < vertex_inputs.chunks(batch_size).len() {
        let buffer = device.create_buffer(
            &BufferDescriptor { 
                label: Some(&format!("{} Vertex Buffer #{}", name, vertex_buffers.len())), 
                size: (batch_size * std::mem::size_of::<T>()) as u64, 
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
                mapped_at_creation: false,
            },
        );
        vertex_buffers.push(buffer);
    }

    render_pass.set_pipeline(&pipeline);
    for (i, bind_group) in bind_groups.iter().enumerate() {
        render_pass.set_bind_group(i as u32, bind_group, &[]);
    }
    for (i, batch) in vertex_inputs.chunks(batch_size).enumerate() {
        let bytes: &[u8] = bytemuck::cast_slice(batch);
        if bytes.len() % (COPY_BUFFER_ALIGNMENT as usize) != 0 {
            todo!("wgpu copy buffer alignment isn't respected. Buffer size={}, desired alignment={}", 
                bytes.len(), 
                COPY_BUFFER_ALIGNMENT);
        }
        queue.write_buffer(&vertex_buffers[i], 0u64, bytes);
        render_pass.set_vertex_buffer(0, vertex_buffers[i].slice(0..(bytes.len() as u64)));
        render_pass.draw(0..((batch.len() * vertexes_per_input) as u32), 0..1);
    }
    
    vertex_inputs.clear();
}