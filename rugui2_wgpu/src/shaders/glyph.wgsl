@group(0)@binding(0) var<uniform> screen_size: vec2<f32>;

@group(2)@binding(0) var t_mask: texture_3d<f32>;
@group(2)@binding(1) var t_sampler: sampler;

const neg_y = vec2(1.0, -1.0);

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uvd: vec3<f32>,
    @location(4) origin: vec2<f32>,
    @location(5) rotation: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uvd: vec3<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
}

override GLYPH_ATLAS_SIDE: f32;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color;

    var half_screen_size = screen_size * 0.5;

    var vert_pos = vertex_position(in.index);
    var glyph_dims = in.size * vert_pos;
    var tex_dims = (in.size / GLYPH_ATLAS_SIDE) * vert_pos;
    var pixel_position = in.position + glyph_dims;

    var origin_relative = pixel_position - in.origin;
    
    var cos_angle = cos(in.rotation);
    var sin_angle = sin(in.rotation);
    var rotated = vec2(
        origin_relative.x * cos_angle - origin_relative.y * sin_angle,
        origin_relative.x * sin_angle + origin_relative.y * cos_angle
    );

    var position = (rotated + in.origin) / half_screen_size - 1.0;


    out.position = vec4(position * neg_y, 0.0, 1.0);
    out.uvd = vec3(tex_dims + in.uvd.xy, in.uvd.z);

    

    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    return vec4(in.color.rgb, in.color.a * textureSample(t_mask, t_sampler, in.uvd).r);
}

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // i: 0 1 2 3 4 5
    // x: + + - - - +
    // y: + - - - + +
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}


//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-  🐓🐓🐓
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-
//MONOLITH-MONOLITH-MONOLITH-MONOLITH-MONOLITH-