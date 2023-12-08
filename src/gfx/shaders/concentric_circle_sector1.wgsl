struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) pixel_xy: vec2<f32>,
    @location(2) center: vec2<f32>,
    @location(3) r1: f32,
    @location(4) r2: f32,
    @location(5) color1: vec4<f32>,
    @location(6) color2: vec4<f32>,
    @location(7) theta_range1: vec2<f32>,
    @location(8) theta_range2: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pixel_xy: vec2<f32>,
    @location(1) @interpolate(flat) center: vec2<f32>,
    @location(2) @interpolate(flat) r1: f32,
    @location(3) @interpolate(flat) r2: f32,
    @location(4) @interpolate(flat) color1: vec4<f32>,
    @location(5) @interpolate(flat) color2: vec4<f32>,
    @location(6) @interpolate(flat) theta_range1: vec2<f32>,
    @location(7) @interpolate(flat) theta_range2: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    out.pixel_xy = input.pixel_xy;
    out.center = input.center;
    out.r1 = input.r1;
    out.r2 = input.r2;
    out.color1 = input.color1;
    out.color2 = input.color2;
    out.theta_range1 = input.theta_range1;
    out.theta_range2 = input.theta_range2;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rel_x: f32 = in.pixel_xy.x - in.center.x;
    // WGSL NDC y is inverted, so we invert y here when calculating atan2
    let rel_y: f32 = -(in.pixel_xy.y - in.center.y);
    let theta: f32 = atan2(rel_y, rel_x);
    if(!((theta >= in.theta_range1[0] && theta <= in.theta_range1[1]) || (theta >= in.theta_range2[0] && theta <= in.theta_range2[1]))) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let hypot2: f32 = rel_x * rel_x + rel_y * rel_y;
    if(hypot2 < in.r1 * in.r1) {
        return in.color1;
    }
    if(hypot2 < in.r2 * in.r2) {
        return in.color2;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}