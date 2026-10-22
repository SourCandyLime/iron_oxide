@group(0) @binding(0)
var framebuffer: texture_2d<u32>;

struct ScreenUniform {
    viewport_origin: vec2<f32>,
    viewport_size: vec2<f32>,
};

@group(0) @binding(1)
var<uniform> screen: ScreenUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );

    return vec4<f32>(
        positions[vertex_index],
        0.0,
        1.0
    );
}

@fragment
fn fs_main(
    @builtin(position) position: vec4<f32>
) -> @location(0) vec4<f32> {
    let framebuffer_size_u = textureDimensions(framebuffer);

    let framebuffer_size = vec2<f32>(
        f32(framebuffer_size_u.x),
        f32(framebuffer_size_u.y),
    );

    // Convert from window coordinates into coordinates
    // relative to the game viewport.
    let local_position =
        position.xy - screen.viewport_origin;

    // Convert viewport position into 0.0 - 1.0 coordinates.
    let uv =
        local_position / screen.viewport_size;

    // Convert UV coordinates into framebuffer pixel coordinates.
    var coord = vec2<i32>(
        uv * framebuffer_size
    );

    // Keep coordinates safely inside the framebuffer.
    coord.x = clamp(
        coord.x,
        0,
        i32(framebuffer_size_u.x) - 1
    );

    coord.y = clamp(
        coord.y,
        0,
        i32(framebuffer_size_u.y) - 1
    );

    // Read the exact RGB332 byte.
    let packed = textureLoad(
        framebuffer,
        coord,
        0
    ).r;

    // Decode RRRGGGBB.
    let r_bits = (packed >> 5u) & 0x7u;
    let g_bits = (packed >> 2u) & 0x7u;
    let b_bits = packed & 0x3u;

    // Convert the integer channels into 0.0 - 1.0 colors.
    let r = f32(r_bits) / 7.0;
    let g = f32(g_bits) / 7.0;
    let b = f32(b_bits) / 3.0;

    return vec4<f32>(
        r,
        g,
        b,
        1.0
    );
}