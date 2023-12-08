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
var hdr_texture: texture_2d<f32>;

@group(0) @binding(1)
var hdr_sampler: sampler;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let hdr = textureSample(hdr_texture, hdr_sampler, vs.xy);
    let luminance = 0.2126 * hdr.r + 0.7152 * hdr.g + 0.0722 * hdr.b;
    if luminance > 1.0 {
        return vec4<f32>((luminance - 1.0) / luminance * hdr.rgb, 1.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
 