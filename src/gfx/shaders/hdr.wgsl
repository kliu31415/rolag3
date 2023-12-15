// Maps HDR values to linear values
// Based on http://www.oscars.org/science-technology/sci-tech-projects/aces
fn hdr_to_linear_aces(hdr: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * hdr;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}

fn hdr_to_linear_simple(hdr: vec3<f32>) -> vec3<f32> {
    return hdr / (1.0 + hdr);
}

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
    let srgb = hdr_to_linear_simple(hdr.rgb);
    return vec4(srgb, hdr.a);
}
 