use wgpu::{Operations, util::DeviceExt};

use crate::gfx::renderer::TextureAndMetadata;

use super::util::new_wgpu_shader_pipeline;

pub struct BloomPipeline {
    extract_bright_pixels_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    addition_pipeline: wgpu::RenderPipeline,

    horizontal_blur_bindgroup: wgpu::BindGroup,
    vertical_blur_bindgroup: wgpu::BindGroup,

    tmd_input: TextureAndMetadata,
    tmd_bright: TextureAndMetadata,
    tmd_blur1: TextureAndMetadata,
    tmd_blur2: TextureAndMetadata,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BloomBlurUniform {
    weights: [f32; 128],
    num_iterations: [u32; 4],
    is_horizontal: [u32; 4],
}

fn normal_pdf(x: f64) -> f64 {
    (1.0 / f64::sqrt(2.0 * std::f64::consts::PI)) * f64::exp(-0.5 * x * x)
}

impl BloomBlurUniform {
    fn new(is_horizontal: bool, num_iterations: u32, std_dev: f64) -> Self {
        assert!(num_iterations < 128, "num_iterations({}) >= 128", num_iterations);
        let mut weights = [0.0; 128];
        let mut weight_sum = 0.0;
        for i in 0..num_iterations {
            let w = normal_pdf(i as f64 / std_dev);
            weight_sum += w;
            weights[i as usize] = w as f32;
        }
        for i in 0..num_iterations {
            weights[i as usize] /= weight_sum as f32;
            println!("{},{}", i, weights[i as usize]);
        }
        Self {
            weights,
            num_iterations: [num_iterations, 0, 0, 0],
            is_horizontal: [is_horizontal as u32, 0, 0, 0],
        }
    }
}

impl BloomPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, width: u32, height: u32) -> Self {
        let tmd_input = TextureAndMetadata::new(device, "bloom_input", format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, width, height);
        let tmd_bright = TextureAndMetadata::new(device, "bloom_bright", format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, width, height);
        let tmd_blur1 = TextureAndMetadata::new(device, "bloom_blur1", format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, width, height);
        let tmd_blur2 = TextureAndMetadata::new(device, "bloom_blur2", format, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT, width, height);
        
        let shader = include_str!("bloom_extract_bright_pixels.wgsl");
        let bg: [&wgpu::BindGroupLayout; 1] = [&TextureAndMetadata::get_standard_bind_group_layout(&device)];
        let extract_bright_pixels_pipeline = new_wgpu_shader_pipeline("bloom_extract_bright_pixels", shader, &device, format, &[], &bg);

        let shader = include_str!("bloom_blur.wgsl");
        let bg = [&TextureAndMetadata::get_standard_bind_group_layout(&device), &Self::get_blur_bind_group_layout(&device)];
        let blur_pipeline = new_wgpu_shader_pipeline("bloom_blur", shader, &device, format, &[], &bg);

        let shader = include_str!("bloom_addition.wgsl");
        let bg = [&TextureAndMetadata::get_standard_bind_group_layout(&device), &TextureAndMetadata::get_standard_bind_group_layout(&device)];
        let addition_pipeline = new_wgpu_shader_pipeline("bloom_addition", shader, &device, format, &[], &bg);

        let blur_uniform_horizontal_cpu = BloomBlurUniform::new(true, 35, 10.0);
        let blur_uniform_horizontal_gpu = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("bloom_blur_horizontal_uniform_buffer"),
                contents: bytemuck::cast_slice(&[blur_uniform_horizontal_cpu]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );
        let horizontal_blur_bindgroup = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &Self::get_blur_bind_group_layout(&device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(blur_uniform_horizontal_gpu.as_entire_buffer_binding()),
                    },
                ],
                label: Some("bloom blur uniform horizontal bind group"),
            }
        );

        let blur_uniform_vertical_cpu = BloomBlurUniform::new(false, 35, 10.0);
        let blur_uniform_vertical_gpu = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("bloom_blur_vertical_uniform_buffer"),
                contents: bytemuck::cast_slice(&[blur_uniform_vertical_cpu]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );
        let vertical_blur_bindgroup = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &Self::get_blur_bind_group_layout(&device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(blur_uniform_vertical_gpu.as_entire_buffer_binding()),
                    },
                ],
                label: Some("bloom blur uniform vertical bind group"),
            }
        );

        Self {
            extract_bright_pixels_pipeline,
            blur_pipeline,
            addition_pipeline,
            horizontal_blur_bindgroup,
            vertical_blur_bindgroup,
            tmd_input,
            tmd_bright,
            tmd_blur1,
            tmd_blur2,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, width: u32, height: u32) {
        *self = Self::new(device, format, width, height);
    }

    pub fn get_input_view(&self) -> &wgpu::TextureView {
        &self.tmd_input.view
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut render_pass = Self::begin_render_pass(encoder, "bloom 1: extract bright pixels", &self.tmd_bright.view);
        render_pass.set_pipeline(&self.extract_bright_pixels_pipeline);
        render_pass.set_bind_group(0, &self.tmd_input.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        let mut render_pass = Self::begin_render_pass(encoder, "bloom 2: horizontal blur", &self.tmd_blur1.view);
        render_pass.set_pipeline(&self.blur_pipeline);
        render_pass.set_bind_group(0, &self.tmd_bright.bind_group, &[]);
        render_pass.set_bind_group(1, &self.horizontal_blur_bindgroup, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        let mut render_pass = Self::begin_render_pass(encoder, "bloom 3: vertical blur", &self.tmd_blur2.view);
        render_pass.set_pipeline(&self.blur_pipeline);
        render_pass.set_bind_group(0, &self.tmd_blur1.bind_group, &[]);
        render_pass.set_bind_group(1, &self.vertical_blur_bindgroup, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);

        let mut render_pass = Self::begin_render_pass(encoder, "bloom 4: addition", output);
        render_pass.set_pipeline(&self.addition_pipeline);
        render_pass.set_bind_group(0, &self.tmd_blur2.bind_group, &[]);
        render_pass.set_bind_group(1, &self.tmd_input.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);
    }

    fn begin_render_pass<'a>(encoder: &'a mut wgpu::CommandEncoder, name: &str, view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a> {
         encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    fn get_blur_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Bloom blur bind group layout"),
        })
    }
}
 