use wgpu::Operations;

use crate::gfx::renderer::TextureAndMetadata;

use super::util::new_wgpu_shader_pipeline;

pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    tmd: TextureAndMetadata,
}

impl HdrPipeline {
    pub fn new(device: &wgpu::Device, intermediate_format: wgpu::TextureFormat, output_format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let tmd = TextureAndMetadata::new(device, "HDR", intermediate_format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, width, height);

        let shader = include_str!("hdr.wgsl");
        let bg = [&TextureAndMetadata::get_standard_bind_group_layout(&device)];
        let pipeline = new_wgpu_shader_pipeline("hdr", shader, &device, output_format, &[], &bg);

        Self {
            pipeline,
            tmd,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, output_format: wgpu::TextureFormat, width: u32, height: u32) {
        let texture = TextureAndMetadata::new(device, "HDR", output_format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, width, height);
        self.tmd = texture;
    }

    pub fn get_input_view(&self) -> &wgpu::TextureView {
        &self.tmd.view
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hdr process"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(Default::default()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.tmd.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
 