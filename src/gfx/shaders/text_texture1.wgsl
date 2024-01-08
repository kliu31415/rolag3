struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    out.color = input.color;
    out.tex_coords = input.tex_coords;
    return out;
}

@group(0) @binding(0)
var texture_: texture_2d<f32>;
@group(0) @binding(1)
var sampler_: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let a = textureSample(texture_, sampler_, in.tex_coords).r;
    return vec4<f32>(in.color.r * a, in.color.g * a, in.color.b * a, in.color.a * a);
}