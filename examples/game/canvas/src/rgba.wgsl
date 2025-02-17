struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @location(0)tex_coords: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    if (in.vertex_index == 0u) {
        out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0);
        out.tex_coords = vec2<f32>(0.0, 1.0);
    } else if (in.vertex_index == 1u) {
        out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0);
        out.tex_coords = vec2<f32>(0.0, 0.0);
    } else if (in.vertex_index == 2u) {
        out.clip_position = vec4<f32>(1.0, -1.0, 0.0, 1.0);
        out.tex_coords = vec2<f32>(1.0, 1.0);
    } else {
        out.clip_position = vec4<f32>(1.0, 1.0, 0.0, 1.0);
        out.tex_coords = vec2<f32>(1.0, 0.0);
    }
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> dimensions: vec2<f32>;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}