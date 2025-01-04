@group(0)@binding(0) var<uniform> screen_size: vec2<f32>;

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) rotation: f32,
    @location(3) color: vec4<f32>,
    @location(4) flags: u32,
    @location(5) round: vec2<f32>,
    @location(6) alpha: f32,
    @location(7) lin_grad_p1p2: vec4<f32>,
    @location(8) lin_grad_p1_color: vec4<f32>,
    @location(9) lin_grad_p2_color: vec4<f32>,
    @location(10) rad_grad_p1p2: vec4<f32>,
    @location(11) rad_grad_p1_color: vec4<f32>,
    @location(12) rad_grad_p2_color: vec4<f32>,
    @location(13) texture_tint: vec4<f32>,
    @location(14) _texture_size: vec2<f32>, // left here for reasosns unknown
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) pixel_size: vec2<f32>,
    @location(1) @interpolate(flat) round: vec2<f32>,
    @location(2) @interpolate(flat) size: vec2<f32>,
}


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate vertex position
    var position = vertex_position(in.index);
    out.size = in.size * 0.5;
    out.round = in.round;

    // Scale and rotate the position
    var scale = in.size * position;
    out.pixel_size = scale;
    var cos_angle = cos(in.rotation);
    var sin_angle = sin(in.rotation);
    var rotated_position = vec2(
        scale.x * cos_angle - scale.y * sin_angle,
        scale.x * sin_angle + scale.y * cos_angle
    );
    
    // Translate to the new position
    var pixel_position = in.position + rotated_position;
    
    // Convert to screen space
    var screen_space = pixel_position / screen_size * 2.0 - 1.0;
    var invert_y = vec2(screen_space.x, -screen_space.y);

    out.position = vec4<f32>(invert_y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    var pos_abs = abs(in.pixel_size);
    if pos_abs.x > (in.size.x - in.round.x) && pos_abs.y > (in.size.y - in.round.x) {
        let the_d = distance(pos_abs, in.size - in.round.x);
        if the_d > in.round.x + in.round.y {
            discard;
        }
    }
    return vec4(0.0);
}

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // i: 0 1 2 3 4 5
    // x: + + - - - +
    // y: + - - - + +
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u))-0.5;
}
