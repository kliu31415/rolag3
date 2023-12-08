struct VertexOutput {
    @location(0) xy: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

fn get_xy(vi: u32) -> vec2<f32> {
    switch(vi) {
        case 0u: {
            return vec2<f32>(0.0, 0.0);
        }
        case 1u: {
            return vec2<f32>(1.0, 0.0);
        }
        case 2u: {
            return vec2<f32>(0.0, 1.0);
        }
        case 3u: {
            return vec2<f32>(1.0, 1.0);
        }
        case 4u: {
            return vec2<f32>(1.0, 0.0);
        }
        case 5u: {
            return vec2<f32>(0.0, 1.0);
        }
        default: {
            // error. Return placeholder.
            return vec2<f32>(0.0, 0.0);
        }
    }
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    let xy: vec2<f32> = get_xy(vi);
    var out: VertexOutput;
    out.clip_position = vec4<f32>(2.0*xy - 1.0, 0.0, 1.0);
    // we need to invert the y coordinate so the image is not upside down
    out.xy = vec2<f32>(xy.x, 1.0 - xy.y);
    return out;
}

@group(0) @binding(0)
var texture_: texture_2d<f32>;

@group(0) @binding(1)
var sampler_: sampler;

struct BlurInfo {
    // we need to use weird types because we need to respect 16-byte alignment constraints
    weights: array<vec4<f32>, 32>,
    num_iterations: vec4<u32>,
    is_horizontal: vec4<u32>,
}

@group(1) @binding(0)
var<uniform> blur_info: BlurInfo;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let step_x = 1.0 / f32(textureDimensions(texture_, 0).x);
    let step_y = 1.0 / f32(textureDimensions(texture_, 0).y);
    
    var col = blur_info.weights[0][0] * textureSample(texture_, sampler_, vs.xy).rgb;
    if(blur_info.is_horizontal[0] == 1u) {
        for(var i = 1u; i <= blur_info.num_iterations[0]; i += 1u) {
            let idx1 = i >> 2u;
            let idx2 = i & 3u;
            let weight = blur_info.weights[idx1][idx2];
            let dist = vec2<f32>(f32(i) * step_x, 0.0);
            col += weight * textureSample(texture_, sampler_, vs.xy + dist).rgb;
            col += weight * textureSample(texture_, sampler_, vs.xy - dist).rgb;
        }
    } else {
        for(var i = 1u; i <= blur_info.num_iterations[0]; i += 1u) {
            let idx1 = i >> 2u;
            let idx2 = i & 3u;
            let weight = blur_info.weights[idx1][idx2];
            let dist = vec2<f32>(0.0, f32(i) * step_y);
            col += weight * textureSample(texture_, sampler_, vs.xy + dist).rgb;
            col += weight * textureSample(texture_, sampler_, vs.xy - dist).rgb;
        }
    }
    return vec4(col, 1.0);
}
 