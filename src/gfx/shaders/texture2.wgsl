struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color_mod: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color_mod: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    out.color_mod = input.color_mod;
    return out;
}

@group(0) @binding(0)
var texture_: texture_2d<f32>;
@group(0) @binding(1)
var sampler_: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture_, sampler_, in.tex_coords);
    return vec4<f32>(color.r * in.color_mod.r, color.g * in.color_mod.g, color.b * in.color_mod.b, color.a * in.color_mod.a);
}