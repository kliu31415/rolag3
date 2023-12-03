use super::shader_pipeline::{new_wgpu_shader_pipeline, draw_triangle_inputs_batched};

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
    bind_groups: Vec<wgpu::BindGroup>,
    vertex_buffers: Vec<wgpu::Buffer>, // GPU memory
    vertex_inputs: Vec<[TextTextureVertexShaderInput; 3]>, // CPU memory,
}

impl TextTextureShaderPipeline {
    const NAME: &'static str = "TextTexture1";
    const BATCH_SIZE: usize = 10;

    pub fn new(
        queue: &wgpu::Queue, 
        device: &wgpu::Device, 
        config: &wgpu::SurfaceConfiguration, 
        bytes: &[u8], 
        width: u32, 
        height: u32
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm, // note there's only one color channel
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("text texture"),
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            texture_size,
        );
    
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
    
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float{ filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("text texture bind group layout"),
        });

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("text texture bind group"),
            }
        );

        let pipeline = new_wgpu_shader_pipeline(
            Self::NAME, 
            include_str!("text_texture1.wgsl").into(), 
            &device, 
            &config, 
            TextTextureVertexShaderInput::desc(),
            &[&texture_bind_group_layout]);

        Self {
            pipeline,
            bind_groups: vec![bind_group],
            vertex_buffers: Vec::new(),
            vertex_inputs: Vec::new(),
        }
    }

    pub fn add_triangle(&mut self, input: &[TextTextureVertexShaderInput; 3]) {
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
            self.bind_groups.as_slice(),
        );
    }
}

