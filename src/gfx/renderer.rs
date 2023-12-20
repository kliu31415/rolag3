use std::{collections::{VecDeque, HashMap, BTreeMap}, rc::Rc, sync::atomic::{AtomicU64, Ordering}, ops::Range};

use wgpu::BufferDescriptor;

use crate::util::time::now_unix;

use super::{shaders::{triangle1::{TriangleVertexShaderInput, TriangleShaderPipeline}, text_texture1::{TextTextureVertexShaderInput, TextTextureShaderPipeline}, concentric_circle_sector1::{ConcrenticCircleSectorShaderPipeline, ConcrenticCircleSectorVertexShaderInput}, hdr::HdrPipeline, bloom::BloomPipeline, texture2::{Texture2ShaderPipeline, Texture2VertexShaderInput}}, text::font::{FontRasterizer, make_font_rasterizer}};

pub trait Renderer {
    fn resize(&mut self, width: u32, height: u32);
    fn get_fps(&self) -> u32;
    fn bytes_to_texture_rgba8888(&mut self, name: &str, bytes: &[u8], width: u32, height: u32) -> TmdRef;

    fn draw(&mut self, op: DrawOpWithMetadata);
    fn present(&mut self, clear_color: ColorRGBA32f) -> Result<(), wgpu::SurfaceError>;
}

#[derive(Debug)]
pub struct DrawOpWithMetadata {
    pub z: f64,
    pub op: DrawOp,
}

impl DrawOpWithMetadata {
    pub fn new(z: f64, op: DrawOp) -> Self {
        Self {z, op}
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq)]
enum ShaderId {
    Triangle1,
    ConcentricCircleSector,
    Text1(String),
    Texture2(TmdId)
}

enum ShaderInput {
    Triangle1(Range<usize>),
    ConcentricCircleSector(Range<usize>),
    Text1((Range<usize>, usize /* bind_group_idx */)),
    Texture2((Range<usize>, usize /* bind_group_idx */)),
}

/* Tri and QuadFan are subsets of TriFan, but they exist to eliminate heap allocation costs. Tri/QuadFan are backed by
   a stack-allocated fixed-size array, while TriFan is backed by a heap-allocated boxed slice.
 */
#[derive(Debug)]
pub enum DrawOp {
    Group(DrawOpGroup),
    _Tri(DrawOpTri),
    QuadFan(DrawOpQuadFan),
    TriFan(DrawOpTriFan),
    _TriStrip(DrawOpTriStrip),
    ConcentricCircleSector(DrawOpCCS),
    Text(DrawOpText),
    Texture2(DrawOpTexture2),
}

impl DrawOp {
    fn get_shader_id(&self) -> ShaderId {
        match self {
            DrawOp::Group(ref g) => g.ops[0].get_shader_id(),
            DrawOp::_Tri(_) => ShaderId::Triangle1,
            DrawOp::QuadFan(_) => ShaderId::Triangle1,
            DrawOp::TriFan(_) => ShaderId::Triangle1,
            DrawOp::_TriStrip(_) => ShaderId::Triangle1,
            DrawOp::ConcentricCircleSector(_) => ShaderId::ConcentricCircleSector,
            DrawOp::Text(ref t) => ShaderId::Text1(t.text.clone()),
            DrawOp::Texture2(ref t) => ShaderId::Texture2(t.texture.id),
        }
    }

    fn flatten(&self) -> Box<dyn Iterator<Item = &DrawOp> + '_> {
        match self {
            DrawOp::Group(ref g) => Box::new(g.ops.iter().flat_map(|x| x.flatten())),
            _ => Box::new(std::iter::once(self)),
        }
    }
}

#[derive(Debug)]
pub struct DrawOpGroup {
    pub ops: Box<[DrawOp]>,
}

impl DrawOpGroup {
    pub fn new(ops: Box<[DrawOp]>) -> Self {
        Self {
            ops
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ColoredTriVertex {
    pub color: ColorRGBA32f,
    pub vertex: ViewSpaceCoordinate,
}

#[derive(Debug)]
pub struct DrawOpTri {
    pub vertexes: [ColoredTriVertex; 3],
}

#[derive(Debug)]
pub struct DrawOpQuadFan {
    pub vertexes: [ColoredTriVertex; 4],
}

#[derive(Debug)]
pub struct DrawOpTriFan {
    pub vertexes: Box<[ColoredTriVertex]>,
}

#[derive(Debug)]
pub struct DrawOpTriStrip {
    pub vertexes: Box<[ColoredTriVertex]>,
}

#[derive(Debug)]
pub struct DrawOpCCS {
    pub x: f32,
    pub y: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub viewport: Option<Rect>,
    pub inner_color: ColorRGBA32f,
    pub outer_color: ColorRGBA32f,
    pub angle_range: Option<(f32, f32)>,
}

#[derive(Debug)]
pub struct DrawOpText {
    pub text: String,
    pub color: ColorRGBA32f, 
    pub x: f32, 
    pub y: f32, 
    pub font_size: f32, 
    pub position: DrawTextPosition,
}

impl DrawOpText {
    fn get_key(&self) -> CachedTextTextureK {
        CachedTextTextureK { text: self.text.clone(), font_size: self.font_size as u32 }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DrawTextPosition {
    TopLeft
}

#[derive(Debug)]
pub struct DrawOpTexture2 {
    pub texture: TmdRef,
    pub color_mod: ColorRGBA32f,
    pub src_rect: Option<Rect>,
    pub dst_rect: Rect,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ColorRGBA32f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRGBA32f {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        ColorRGBA32f { r, g, b, a}
    }
    pub fn to_f32x4(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {x, y, w, h}
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ViewSpaceCoordinate {
    pub x: f32,
    pub y: f32,
}

impl ViewSpaceCoordinate {
    pub fn new(x: f32, y: f32) -> Self {
        ViewSpaceCoordinate { x, y }
    }
}

struct WgpuRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    frame_timestamps: VecDeque<f64>,

    draw_ops: Vec<DrawOpWithMetadata>,

    vertex_buffer_pool: VertexBufferPool,
    triangle_shader_pipeline: TriangleShaderPipeline,
    concentric_circle_sector_shader_pipeline: ConcrenticCircleSectorShaderPipeline,
    text_shader_pipeline: TextTextureShaderPipeline,
    texture2_shader_pipeline: Texture2ShaderPipeline,
    bloom_pipeline: BloomPipeline,
    hdr_pipeline: HdrPipeline,

    cached_text_textures: HashMap<CachedTextTextureK, CachedTextTextureV>,
    textures: HashMap<TmdId, TextureAndMetadata>, // TODO: remove entries of this map with 0 external references
    font_rasterizer: Box<dyn FontRasterizer>,

    per_frame_allocator: bumpalo::Bump,
    cached_vecs: WgpuRendererCachedVecs,
}

struct WgpuRendererCachedVecs {
    shader_input_batches: Vec<ShaderInput>,
    triangle1_shader_inputs_batch: Vec<[TriangleVertexShaderInput; 3]>,
    ccs_inputs_batch: Vec<[ConcrenticCircleSectorVertexShaderInput; 3]>,
    text_inputs_batch: Vec<[TextTextureVertexShaderInput; 3]>,
    texture2_inputs_batch: Vec<[Texture2VertexShaderInput; 3]>,
    desired_buffer_sizes: Vec<usize>,
}

impl WgpuRendererCachedVecs {
    fn new() -> Self {
        Self {
            shader_input_batches: Vec::new(),
            triangle1_shader_inputs_batch: Vec::new(),
            ccs_inputs_batch: Vec::new(),
            text_inputs_batch: Vec::new(),
            texture2_inputs_batch: Vec::new(),
            desired_buffer_sizes: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.shader_input_batches.clear();
        self.triangle1_shader_inputs_batch.clear();
        self.ccs_inputs_batch.clear();
        self.text_inputs_batch.clear();
        self.texture2_inputs_batch.clear();
        self.desired_buffer_sizes.clear();
    }
}

#[derive(Debug)]
pub struct TextureAndMetadata {
    pub tmd_ref: TmdRef,
    pub name: String,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub sampler: wgpu::Sampler,
}

pub type TmdId = u64;

#[derive(Debug, Clone)]
pub struct TmdRef {
    rc: Rc<()>, // when a texture's strong ref count reaches 1, no external clients have access to it, so it's deleted
    id: TmdId,
}

impl TmdRef {
    fn new() -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            rc: Rc::new(()),
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

}

#[derive(Debug)]
struct CachedTextTextureV {
    tmd: TextureAndMetadata,
    last_used: f64,
}

impl TextureAndMetadata {
    pub fn get_standard_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("TextureAndMetadata bind group layout"),
        })
    }

    pub fn new(
        device: &wgpu::Device, 
        name: &str, 
        format: wgpu::TextureFormat, 
        usage: wgpu::TextureUsages,
        width: u32, 
        height: u32,
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
                format, 
                usage,
                label: Some(&format!("texture {}", name)),
                view_formats: &[],
            }
        );
    
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &Self::get_standard_bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some(&format!("texture bind group {}", name)),
            }
        );

        Self {
            tmd_ref: TmdRef::new(),
            name: name.to_owned(),
            texture,
            view,
            bind_group,
            sampler,
        }
    }

    fn write_bytes(&mut self, queue: &wgpu::Queue, bytes_per_pixel: u32, bytes: &[u8]) {
        let texture_size = wgpu::Extent3d {
            width: self.texture.width(),
            height: self.texture.height(),
            depth_or_array_layers: 1,
        };
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.texture.width() * bytes_per_pixel),
                rows_per_image: Some(self.texture.height()),
            },
            texture_size,
        );
    }

    fn get_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &Self::get_standard_bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some(&format!("texture bind group {}", self.name)),
            }
        )
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CachedTextTextureK {
    text: String,
    font_size: u32,
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.hdr_pipeline.resize( &self.device, Self::INTERMEDIATE_TEXTURE_FORMAT, width, height);
            self.bloom_pipeline.resize(&self.device, Self::INTERMEDIATE_TEXTURE_FORMAT, width, height);
        }
    }

    fn get_fps(&self) -> u32 {
        self.frame_timestamps.len() as u32
    }

    fn draw(&mut self, op: DrawOpWithMetadata) {
        self.process_for_draw_op(&op.op);
        self.draw_ops.push(op);
    }

    fn present(&mut self, clear_color: ColorRGBA32f) -> Result<(), wgpu::SurfaceError> {
        self.per_frame_allocator.reset();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut cached_mem = WgpuRendererCachedVecs::new();
        std::mem::swap(&mut cached_mem, &mut self.cached_vecs);
        cached_mem.reset();
        let mut bind_groups = Vec::new();
        let buffers: Vec<_>;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.bloom_pipeline.get_input_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: clear_color.r as f64,
                                g: clear_color.g as f64,
                                b: clear_color.b as f64,
                                a: clear_color.a as f64,
                            }
                        ),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // sort DrawOps by z. Order them in a way that minimizes the amount of times the shader pipeline is changed
            self.draw_ops.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
            let mut ops_per_z = vec![Vec::new()];
            let mut ops_this_z = Vec::new();
            let mut prev_z = f64::NEG_INFINITY;
            for op in self.draw_ops.iter() {
                if op.z != prev_z {
                    ops_per_z.push(ops_this_z);
                    ops_this_z = Vec::new();
                    prev_z = op.z;
                }
                ops_this_z.push(&op.op);
            }
            if !ops_this_z.is_empty() {
                ops_per_z.push(ops_this_z);
            }
            for (i, ops) in ops_per_z.iter_mut().enumerate() {
                ops.sort_by_key(|x| x.get_shader_id());
                if i % 2 == 0 {
                    ops.reverse();
                }
            }

            // convert DrawOps into GPU shader inputs
            let mut ordered_ops = ops_per_z.drain(..).flatten().flat_map(|x| x.flatten()).peekable();
            let mut prev_triangle1_shader_inputs_len = 0;
            let mut prev_ccs_shader_inputs_len = 0;
            let mut prev_text_shader_inputs_len = 0;
            let mut prev_texture2_shader_inputs_len = 0;

            let mut text_inputs_batch_bg = None;
            let mut texture2_inputs_batch_bg = None;
            while let Some(op) = ordered_ops.next() {
                match op {
                    DrawOp::Group(_) => panic!("all DrawOpGroups should have been flattened by this point (1)"),
                    DrawOp::_Tri(ref x) => {
                        self.add_tri_shader_inputs(&mut cached_mem.triangle1_shader_inputs_batch, x);
                    }
                    DrawOp::QuadFan(ref x) => {
                        self.add_quad_fan_shader_inputs(&mut cached_mem.triangle1_shader_inputs_batch, x);
                    }
                    DrawOp::TriFan(ref x) => {
                        self.add_tri_fan_shader_inputs(&mut cached_mem.triangle1_shader_inputs_batch, x);
                    }
                    DrawOp::_TriStrip(ref x) => {
                        self.add_tri_strip_shader_inputs(&mut cached_mem.triangle1_shader_inputs_batch, x);
                    }
                    DrawOp::ConcentricCircleSector(ref x) => {
                        self.add_ccs_shader_inputs(&mut cached_mem.ccs_inputs_batch, x);
                    }
                    DrawOp::Text(ref x) => {
                        let dt = self.add_text_shader_inputs(&mut cached_mem.text_inputs_batch, x);
                        text_inputs_batch_bg = Some(dt);
                    }
                    DrawOp::Texture2(ref x) => {
                        let dt = self.add_texture2_shader_inputs(&mut cached_mem.texture2_inputs_batch, x);
                        texture2_inputs_batch_bg = Some(dt);
                    }
                }
                if ordered_ops.peek().is_none() || op.get_shader_id() != ordered_ops.peek().unwrap().get_shader_id() {
                    match op.get_shader_id() {
                        ShaderId::Triangle1 => {
                            let cur_len = cached_mem.triangle1_shader_inputs_batch.len();
                            let prev_len = prev_triangle1_shader_inputs_len;
                            cached_mem.desired_buffer_sizes.push(std::mem::size_of::<[TriangleVertexShaderInput; 3]>() * (cur_len - prev_len));
                            cached_mem.shader_input_batches.push(ShaderInput::Triangle1(prev_len..cur_len));
                            prev_triangle1_shader_inputs_len = cur_len;
                        }
                        ShaderId::ConcentricCircleSector => {
                            let cur_len = cached_mem.ccs_inputs_batch.len();
                            let prev_len = prev_ccs_shader_inputs_len;
                            cached_mem.desired_buffer_sizes.push(std::mem::size_of::<[ConcrenticCircleSectorVertexShaderInput; 3]>() * (cur_len - prev_len));
                            cached_mem.shader_input_batches.push(ShaderInput::ConcentricCircleSector(prev_len..cur_len));
                            prev_ccs_shader_inputs_len = cur_len;
                        }
                        ShaderId::Text1(_) => {
                            let cur_len = cached_mem.text_inputs_batch.len();
                            let prev_len = prev_text_shader_inputs_len;
                            cached_mem.desired_buffer_sizes.push(std::mem::size_of::<[TextTextureVertexShaderInput; 3]>() * (cur_len - prev_len));
                            let bgi = bind_groups.len();
                            bind_groups.push(vec![text_inputs_batch_bg.unwrap()]);
                            cached_mem.shader_input_batches.push(ShaderInput::Text1((prev_len..cur_len, bgi)));
                            text_inputs_batch_bg = None;
                            prev_text_shader_inputs_len = cur_len;
                        }
                        ShaderId::Texture2(_) => {
                            let cur_len = cached_mem.texture2_inputs_batch.len();
                            let prev_len = prev_texture2_shader_inputs_len;
                            cached_mem.desired_buffer_sizes.push(std::mem::size_of::<[Texture2VertexShaderInput; 3]>() * (cur_len - prev_len));
                            let bgi = bind_groups.len();
                            bind_groups.push(vec![texture2_inputs_batch_bg.unwrap()]);
                            cached_mem.shader_input_batches.push(ShaderInput::Texture2((prev_len..cur_len, bgi)));
                            texture2_inputs_batch_bg = None;
                            prev_texture2_shader_inputs_len = cur_len;
                        }
                    }
                }
            }
            
            // send GPU shader inputs to the GPU and execute shaders
            let desired_buffer_sizes: Vec<_> = cached_mem.desired_buffer_sizes.drain(..).map(|x| x as u64).collect();
            buffers = self.vertex_buffer_pool.allocate(&self.device, &desired_buffer_sizes);
            for (i, batch) in cached_mem.shader_input_batches.drain(..).enumerate() {
                match batch {
                    ShaderInput::Triangle1(x) => {
                        self.triangle_shader_pipeline.draw(&mut render_pass, &self.queue, buffers[i].as_ref(), &cached_mem.triangle1_shader_inputs_batch[x]);
                    }
                    ShaderInput::ConcentricCircleSector(x) => {
                        self.concentric_circle_sector_shader_pipeline.draw(&mut render_pass, &self.queue, buffers[i].as_ref(), &cached_mem.ccs_inputs_batch[x]);
                    }
                    ShaderInput::Text1((x, bgi)) => {
                        self.text_shader_pipeline.draw(&mut render_pass, &self.queue, buffers[i].as_ref(), &cached_mem.text_inputs_batch[x], &bind_groups[bgi]);
                    }
                    ShaderInput::Texture2((x, bgi)) => {
                        self.texture2_shader_pipeline.draw(&mut render_pass, &self.queue, buffers[i].as_ref(), &cached_mem.texture2_inputs_batch[x], &bind_groups[bgi]);
                    }
                }
            }
        }
        std::mem::swap(&mut cached_mem, &mut self.cached_vecs);

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.bloom_pipeline.process(&mut encoder, self.hdr_pipeline.get_input_view());
        self.hdr_pipeline.process(&mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));

        let now = now_unix();
        self.frame_timestamps.push_back(now);
        while !self.frame_timestamps.is_empty() && *self.frame_timestamps.front().unwrap() < now - 1.0 {
            self.frame_timestamps.pop_front();
        }

        output.present();

        self.textures.retain(|_, v| Rc::strong_count(&v.tmd_ref.rc) > 1);
        self.cached_text_textures.retain(|_, v| now - v.last_used < 1.0);
        self.draw_ops.clear();

        Ok(())
    }

    fn bytes_to_texture_rgba8888(&mut self, name: &str, bytes: &[u8], width: u32, height: u32) -> TmdRef {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let usages = wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING;
        let mut tmd = TextureAndMetadata::new(&self.device, name, format, usages, width, height);
        tmd.write_bytes(&self.queue, 4, bytes);
        let tmd_ref = tmd.tmd_ref.clone();
        self.textures.insert(tmd.tmd_ref.id, tmd);
        tmd_ref
    }
}

struct VertexBufferPool {
    buffers: Vec<Rc<wgpu::Buffer>>,
}

impl VertexBufferPool {
    fn new() -> Self {
        Self {buffers: Vec::new()}
    }
    fn allocate(&mut self, device: &wgpu::Device, desired_buffer_sizes: &Vec<u64>) -> Vec<Rc<wgpu::Buffer>> {
        // TODO: clean up old buffers that haven't been used for a while
        self.buffers.sort_by_key(|x| x.size());
        self.buffers.reverse();

        let mut dbs_sorted = desired_buffer_sizes.clone();
        dbs_sorted.sort();
        dbs_sorted.reverse();

        let mut buffers_to_use = BTreeMap::new();
        let mut existing_buffer_idx = 0;
        let mut new_buffers = Vec::new();
        for dbs in dbs_sorted {
            if existing_buffer_idx == self.buffers.len() || self.buffers[existing_buffer_idx].size() < dbs {
                // allocate 1.2x the size of the requested size. This is so that if in consecutive frames, the largest
                // buffer requested slowly increases like 500, 501, 502, 503, etc., we don't allocate a new buffer
                // every time
                let new_buffer_size = (1.2 * (dbs as f64)) as u64;
                let new_buffer = Rc::new(Self::make_vertex_buffer(self.buffers.len(), new_buffer_size, device));
                new_buffers.push(new_buffer.clone());
                if !buffers_to_use.contains_key(&dbs) {
                    buffers_to_use.insert(dbs, Vec::new());
                }
                buffers_to_use.get_mut(&dbs).unwrap().push(new_buffer);
            } else {
                if !buffers_to_use.contains_key(&dbs) {
                    buffers_to_use.insert(dbs, Vec::new());
                }
                buffers_to_use.get_mut(&dbs).unwrap().push(self.buffers[existing_buffer_idx].clone());
                existing_buffer_idx += 1;
            }
        }
        new_buffers.drain(..).for_each(|x| self.buffers.push(x));

        let mut ret = Vec::new();
        for dbs in desired_buffer_sizes {
            let (size, buffers)= buffers_to_use.range_mut(dbs..).next().unwrap();
            ret.push(buffers.pop().unwrap());
            if buffers.is_empty() {
                let sz = *size;
                buffers_to_use.remove(&sz);
            }
        }      
        ret
    }

    fn make_vertex_buffer(idx: usize, size: u64, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(
            &BufferDescriptor { 
                label: Some(&format!("{} Vertex Buffer", idx)), 
                size, 
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
                mapped_at_creation: false,
            },
        )
    }
}

impl WgpuRenderer {
    const INTERMEDIATE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    fn x_to_ndc(&self, x: f32) -> f32 {
        2.0 * x / (self.config.width as f32) - 1.0
    }

    fn y_to_ndc(&self, y: f32) -> f32 {
        1.0 - 2.0 * y / (self.config.height as f32)
    }

    fn w_to_ndc(&self, w: f32) -> f32 {
        2.0 * w / (self.config.width as f32)
    }

    fn h_to_ndc(&self, h: f32) -> f32 {
        2.0 * h / (self.config.height as f32)
    }

    fn process_for_draw_op(&mut self, op: &DrawOp) {
        match op {
            DrawOp::Text(ref t) => {
                let font_size = t.font_size as u32; // we round down to the nearest int for now
                let k = t.get_key();
        
                if !self.cached_text_textures.contains_key(&k) {
                    let bytes_2d = self.font_rasterizer.rasterize_text_line(&t.text, font_size as f32);
                    if bytes_2d.is_empty() {
                        panic!("rasterized 0 bytes while drawing text. Function call draw(args={:?})", op);
                    }
                    let width = bytes_2d[0].len() as u32;
                    let height = bytes_2d.len() as u32;
                    let bytes_1d: Vec<u8> = bytes_2d.into_iter().flatten().collect();
                    let name = format!("text={}", t.text);
                    let mut texture_and_md = TextureAndMetadata::new(&self.device, &name, wgpu::TextureFormat::R8Unorm, wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, width, height);
                    texture_and_md.write_bytes(&self.queue, 1, &bytes_1d);
        
                    let v = CachedTextTextureV {
                        tmd: texture_and_md,
                        last_used: now_unix(),
                    };
                    self.cached_text_textures.insert(k, v);
                } else {
                    let cached_v = self.cached_text_textures.get_mut(&k).expect("unable to get cached text pipeline (1)");
                    cached_v.last_used = now_unix();
                }
            },
            DrawOp::Group(ref g) => {
                g.ops.iter().for_each(|x| self.process_for_draw_op(x));
            }
            _ => {},
        }
    }

    fn add_tri_shader_inputs(&self, dst: &mut Vec<[TriangleVertexShaderInput; 3]>, op: &DrawOpTri) {
        dst.push([self.tri_to_gpu(&op.vertexes[0]), self.tri_to_gpu(&op.vertexes[1]), self.tri_to_gpu(&op.vertexes[2])]);
    }

    fn add_quad_fan_shader_inputs(&self, dst: &mut Vec<[TriangleVertexShaderInput; 3]>, op: &DrawOpQuadFan) {
        dst.push([self.tri_to_gpu(&op.vertexes[0]), self.tri_to_gpu(&op.vertexes[1]), self.tri_to_gpu(&op.vertexes[2])]);
        dst.push([self.tri_to_gpu(&op.vertexes[0]), self.tri_to_gpu(&op.vertexes[2]), self.tri_to_gpu(&op.vertexes[3])]);
    }

    fn add_tri_fan_shader_inputs(&self, dst: &mut Vec<[TriangleVertexShaderInput; 3]>, op: &DrawOpTriFan) {
        if op.vertexes.len() < 3 {
            panic!("draw_tri_fan() expected at least 3 vertexes, got {}. vertexes={:?}", 
                op.vertexes.len(),
                op.vertexes);
        }
        for i in 2..op.vertexes.len() {
            dst.push([self.tri_to_gpu(&op.vertexes[0]), 
                self.tri_to_gpu(&op.vertexes[i-1]), 
                self.tri_to_gpu(&op.vertexes[i])]);
        }
    }

    fn add_tri_strip_shader_inputs(&self, dst: &mut Vec<[TriangleVertexShaderInput; 3]>, op: &DrawOpTriStrip) {
        if op.vertexes.len() < 3 {
            panic!("draw_tri_strip() expected at least 3 vertexes, got {}. vertexes={:?}", 
                op.vertexes.len(), 
                op.vertexes);
        }
        for i in 2..op.vertexes.len() {
            dst.push([self.tri_to_gpu(&op.vertexes[i-2]),
                self.tri_to_gpu(&op.vertexes[i-1]),
                self.tri_to_gpu(&op.vertexes[i])]);
        }
    }

    fn tri_to_gpu(&self, v: &ColoredTriVertex) -> TriangleVertexShaderInput {
        TriangleVertexShaderInput{
            position: [self.x_to_ndc(v.vertex.x), self.y_to_ndc(v.vertex.y)], 
            color: [v.color.r, v.color.g, v.color.b, v.color.a],
        } 
    }

    fn add_ccs_shader_inputs(&self, dst: &mut Vec<[ConcrenticCircleSectorVertexShaderInput; 3]>, args: &DrawOpCCS) {
        if args.inner_radius < 0.0 || args.outer_radius < 0.0 || args.inner_radius > args.outer_radius {
            panic!("concentric circle sector inner_radius({}) and outer_radius({}) have bad values", args.inner_radius, args.outer_radius);
        }
        let full_viewport = Rect::new(args.x - args.outer_radius, args.y - args.outer_radius, args.outer_radius*2.0, args.outer_radius*2.0);
        let viewport = match args.viewport {
            Some(v) => v,
            None => full_viewport,
        };

        let position_x = self.x_to_ndc(viewport.x);
        let position_y = self.y_to_ndc(viewport.y);
        let position_w = self.w_to_ndc(viewport.w);
        let position_h = self.h_to_ndc(viewport.h);

        const DUMMY_ANGLE: f32 = 10.0; // in WGSL, atan2 can never return 10
        let (theta_range1, theta_range2) = match args.angle_range {
            Some((mut angle_begin, mut angle_end)) => (|| {
                if angle_begin == 0.0 && angle_end == 2.0*std::f32::consts::PI {
                    return ([-100.0, 100.0 + 2.0*std::f32::consts::PI], [DUMMY_ANGLE, DUMMY_ANGLE]);
                }
                if !(0.0..=2.0*std::f32::consts::PI).contains(&angle_begin) {
                    panic!("concentric circle sector angle_begin between isn't between 0 and 2*PI. Got {}", angle_begin);
                }
                if !(0.0..=2.0*std::f32::consts::PI).contains(&angle_end) {
                    panic!("concentric circle sector angle_end between isn't between 0 and 2*PI. Got {}", angle_end);
                }

                if angle_begin > std::f32::consts::PI {
                    angle_begin -= 2.0 * std::f32::consts::PI;
                }
                if angle_end > std::f32::consts::PI {
                    angle_end -= 2.0 * std::f32::consts::PI;
                }

                if angle_end < angle_begin {
                    ([angle_begin, std::f32::consts::PI], [-std::f32::consts::PI, angle_end])
                } else {
                    ([angle_begin, angle_end], [DUMMY_ANGLE, DUMMY_ANGLE])
                }
            })(),
            None => ([-100.0, 100.0 + 2.0*std::f32::consts::PI], [DUMMY_ANGLE, DUMMY_ANGLE]),
        };

        let mut vertexes = Vec::new();
        for (dx, dy) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
            vertexes.push(ConcrenticCircleSectorVertexShaderInput {
                // we use + for x and - for y because the y axis in WGSL NDC is inverted
                position: [position_x + dx * position_w, position_y - dy * position_h],
                pixel_xy: [viewport.x + dx * viewport.w, viewport.y + dy * viewport.h],
                center: [args.x, args.y],
                r1: args.inner_radius,
                r2: args.outer_radius,
                color1: args.inner_color.to_f32x4(),
                color2: args.outer_color.to_f32x4(),
                theta_range1,
                theta_range2,
            });
        }

        dst.push([vertexes[0], vertexes[1], vertexes[2]]);
        dst.push([vertexes[2], vertexes[3], vertexes[0]]);
    }

    fn add_text_shader_inputs(&self, dst: &mut Vec<[TextTextureVertexShaderInput; 3]>, args: &DrawOpText) -> wgpu::BindGroup {
        let k = args.get_key();
        let v = self.cached_text_textures.get(&k).expect("unable to get cached text texture");
        
        let (x, y) = match args.position {
            DrawTextPosition::TopLeft => (args.x, args.y),
        };

        let width = v.tmd.texture.width() as f32;
        let height = v.tmd.texture.height() as f32;

        let vertexes = &[
            self.text_tri_to_gpu(&args.color, &ViewSpaceCoordinate::new(x, y), [0.0, 0.0]),
            self.text_tri_to_gpu(&args.color, &ViewSpaceCoordinate::new(x + width, y), [1.0, 0.0]),
            self.text_tri_to_gpu(&args.color, &ViewSpaceCoordinate::new(x + width, y + height), [1.0, 1.0]),
            self.text_tri_to_gpu(&args.color, &ViewSpaceCoordinate::new(x, y + height), [0.0, 1.0]),
        ];

        dst.push([vertexes[0], vertexes[1], vertexes[2]]);
        dst.push([vertexes[2], vertexes[3], vertexes[0]]);
        v.tmd.get_bind_group(&self.device)
    }

    fn text_tri_to_gpu(&self, color: &ColorRGBA32f, v: &ViewSpaceCoordinate, tex_coords: [f32; 2]) -> TextTextureVertexShaderInput {
        TextTextureVertexShaderInput{
            position: [self.x_to_ndc(v.x), self.y_to_ndc(v.y)], 
            color: [color.r, color.g, color.b, color.a],
            tex_coords,
        } 
    }

    fn add_texture2_shader_inputs(&self, dst: &mut Vec<[Texture2VertexShaderInput; 3]>, args: &DrawOpTexture2) -> wgpu::BindGroup {
        let k = args.texture.id;
        let v = self.textures.get(&k).expect("unable to get cached texture");

        let vertexes = &[
            self.texture2_tri_to_gpu(&args.color_mod, &ViewSpaceCoordinate::new(args.dst_rect.x, args.dst_rect.y), [0.0, 0.0]),
            self.texture2_tri_to_gpu(&args.color_mod, &ViewSpaceCoordinate::new(args.dst_rect.x + args.dst_rect.w, args.dst_rect.y), [1.0, 0.0]),
            self.texture2_tri_to_gpu(&args.color_mod, &ViewSpaceCoordinate::new(args.dst_rect.x + args.dst_rect.w, args.dst_rect.y + args.dst_rect.h), [1.0, 1.0]),
            self.texture2_tri_to_gpu(&args.color_mod, &ViewSpaceCoordinate::new(args.dst_rect.x, args.dst_rect.y + args.dst_rect.h), [0.0, 1.0]),
        ];

        dst.push([vertexes[0], vertexes[1], vertexes[2]]);
        dst.push([vertexes[2], vertexes[3], vertexes[0]]);
        v.get_bind_group(&self.device)
    }

    fn texture2_tri_to_gpu(&self, color_mod: &ColorRGBA32f, v: &ViewSpaceCoordinate, tex_coords: [f32; 2]) -> Texture2VertexShaderInput {
        Texture2VertexShaderInput{
            position: [self.x_to_ndc(v.x), self.y_to_ndc(v.y)], 
            tex_coords,
            color_mod: [color_mod.r, color_mod.g, color_mod.b, color_mod.a],
        } 
    }
}

pub fn make_renderer(window: &winit::window::Window) -> Box<dyn Renderer> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe {instance.create_surface(&window)}.unwrap();

    let adapter = pollster::block_on(
        instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        )
    ).unwrap();

    let (device, queue) = pollster::block_on(
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
        None,
        )
    ).unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let vertex_buffer_pool = VertexBufferPool::new();
    let format = WgpuRenderer::INTERMEDIATE_TEXTURE_FORMAT;
    let bloom_pipeline = BloomPipeline::new(&device, format, config.width, config.height);
    let hdr_pipeline = HdrPipeline::new(&device, format, config.format, config.width, config.height);
    let triangle_shader_pipeline = TriangleShaderPipeline::new(&device, format);
    let concentric_circle_sector_shader_pipeline = ConcrenticCircleSectorShaderPipeline::new(&device, format);
    let text_shader_pipeline = TextTextureShaderPipeline::new(&device, format);
    let texture2_shader_pipeline = Texture2ShaderPipeline::new(&device, format);

    let renderer = WgpuRenderer {
        surface,
        device,
        queue,
        config,
        frame_timestamps: VecDeque::new(),
        draw_ops: Vec::new(),
        vertex_buffer_pool,
        triangle_shader_pipeline,
        concentric_circle_sector_shader_pipeline,
        text_shader_pipeline,
        texture2_shader_pipeline,
        bloom_pipeline,
        hdr_pipeline,
        cached_text_textures: HashMap::new(),
        textures: HashMap::new(),
        font_rasterizer: make_font_rasterizer(),
        per_frame_allocator: bumpalo::Bump::with_capacity(1<<27 /* 128 MB */),
        cached_vecs: WgpuRendererCachedVecs::new(),
    };

    Box::new(renderer)
}