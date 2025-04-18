@group(0)@binding(0) var<uniform> screen_size: vec2<f32>;

@group(1)@binding(0) var t_diffuse: texture_2d<f32>;
@group(1)@binding(1) var t_sampler: sampler;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) rotation: f32,
    @location(3) color: vec4<f32>,
    @location(4) flags: u32,
    @location(5) round: f32,
    @location(6) shadow: f32,
    @location(7) alpha: f32,
    @location(8) lin_grad_p1p2: vec4<f32>,
    @location(9) lin_grad_p1_color: vec4<f32>,
    @location(10) lin_grad_p2_color: vec4<f32>,
    @location(11) rad_grad_p1p2: vec4<f32>,
    @location(12) rad_grad_p1_color: vec4<f32>,
    @location(13) rad_grad_p2_color: vec4<f32>,
    @location(14) texture_tint: vec4<f32>,
    @location(15) shadow_alpha: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) pixel_size: vec2<f32>,
    @location(1) pixel_pos: vec2<f32>,
    @location(2) @interpolate(flat) color: vec4<f32>,
    @location(3) @interpolate(flat) flags: u32,
    @location(4) @interpolate(flat) size: vec2<f32>,
    @location(5) @interpolate(flat) round: f32,
    @location(6) @interpolate(flat) shadow: f32,
    @location(7) @interpolate(flat) alpha: f32,
    @location(8) @interpolate(flat) lin_grad_p1p2: vec4<f32>,
    @location(9) @interpolate(flat) lin_grad_p1_color: vec4<f32>,
    @location(10) @interpolate(flat) lin_grad_p2_color: vec4<f32>,
    @location(11) @interpolate(flat) rad_grad_p1p2: vec4<f32>,
    @location(12) @interpolate(flat) rad_grad_p1_color: vec4<f32>,
    @location(13) @interpolate(flat) rad_grad_p2_color: vec4<f32>,
    @location(14) @interpolate(flat) texture_tint: vec4<f32>,
    @location(15) @interpolate(flat) shadow_alpha: f32,
    @location(16) uv: vec2<f32>,
}


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var size_wshadow = in.size + in.shadow*2.0;

    // Calculate vertex position
    var position = vertex_position(in.index);
    out.uv = position + 0.5;
    out.size = size_wshadow * 0.5;

    out.round = in.round;
    out.shadow = in.shadow;
    out.alpha = in.alpha;
    out.color = in.color;
    out.flags = in.flags;
    out.lin_grad_p1p2 = in.lin_grad_p1p2;
    out.lin_grad_p1_color = in.lin_grad_p1_color;
    out.lin_grad_p2_color = in.lin_grad_p2_color;
    out.rad_grad_p1p2 = in.rad_grad_p1p2;
    out.rad_grad_p1_color = in.rad_grad_p1_color;
    out.rad_grad_p2_color = in.rad_grad_p2_color;
    out.texture_tint = in.texture_tint;
    out.shadow_alpha = in.shadow_alpha;

    // Scale and rotate the position
    var scale = size_wshadow * position;
    out.pixel_size = scale;
    var cos_angle = cos(in.rotation);
    var sin_angle = sin(in.rotation);
    var rotated_position = vec2(
        scale.x * cos_angle - scale.y * sin_angle,
        scale.x * sin_angle + scale.y * cos_angle
    );
    
    // Translate to the new position
    var pixel_position = in.position + rotated_position;
    out.pixel_pos = pixel_position;
    
    // Convert to screen space
    var screen_space = pixel_position / screen_size * 2.0 - 1.0;
    var invert_y = vec2(screen_space.x, -screen_space.y);

    out.position = vec4<f32>(invert_y, 0.0, 1.0);

    return out;
}

override LIN_GRADIENT: u32;
override RAD_GRADIENT: u32;
override TEXTURE: u32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    const gamma_exp = 1.0;
    const gamma_mul = 1.0;
    var color = vec3(0.0);
    var max_alpha = 0.0;
    if bool(in.flags & TEXTURE) {
        var c = textureSample(t_diffuse, t_sampler, in.uv) * in.texture_tint;
        color = mix(color, c.rgb, c.a);
        max_alpha = max(max_alpha, c.a);
    }
    if bool(in.flags & RAD_GRADIENT) {
        var p1 = in.rad_grad_p1p2.rg;
        var p2 = in.rad_grad_p1p2.ba;
        var p1color = in.rad_grad_p1_color;
        var p2color = in.rad_grad_p2_color;
        
        var c = mix(p1color, p2color, distance(p1, in.pixel_pos) / distance(p1, p2));
        color = mix(color, c.rgb, c.a);
        max_alpha = max(max_alpha, c.a);
    }
    if bool(in.flags & LIN_GRADIENT) {
        var p1 = in.lin_grad_p1p2.rg;
        var p2 = in.lin_grad_p1p2.ba;
        var p1color = in.lin_grad_p1_color;
        var p2color = in.lin_grad_p2_color;

        var gradient_factor = dot(in.pixel_pos - p1, p2 - p1) / dot(p2 - p1, p2 - p1);
        var c = mix(p1color, p2color, clamp(gradient_factor, 0.0, 1.0));
        color = mix(color, c.rgb, c.a);
        max_alpha = max(max_alpha, c.a);
    }
    color = mix(color, in.color.rgb, in.color.a);
    max_alpha = max(max_alpha, in.color.a);
    var pos_abs = abs(in.pixel_size);
    if pos_abs.x > (in.size.x - in.round - in.shadow) && pos_abs.y > (in.size.y - in.round - in.shadow) {
        let the_d = distance(pos_abs, in.size - in.round - in.shadow);
        let new_alpha = clamp(1.0-((the_d - in.round) / in.shadow), 0.0, 1.0);
        if new_alpha == 1.0 {
            max_alpha *= new_alpha;
        } else {
            max_alpha *= new_alpha * in.shadow_alpha;
        }
    } else if pos_abs.x > (in.size.x - in.shadow) {
        max_alpha *= (1.0 - (pos_abs.x - in.size.x + in.shadow) / (in.shadow)) * in.shadow_alpha;
    } else if pos_abs.y > (in.size.y- in.shadow) {
        max_alpha *= (1.0 - (pos_abs.y - in.size.y + in.shadow) / (in.shadow)) * in.shadow_alpha;
    }
    return vec4(pow(color, vec3(gamma_exp))*gamma_mul, max_alpha * in.alpha);
}

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // i: 0 1 2 3 4 5
    // x: + + - - - +
    // y: + - - - + +
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u))-0.5;
}
