@group(0) @binding(0)
var framebuffer: texture_2d<u32>;

struct ScreenUniform {
    viewport_origin: vec2<f32>,
    viewport_size: vec2<f32>,
};
@group(0) @binding(1)
var<uniform> screen: ScreenUniform;

struct CrtUniform {
    controls: vec4<f32>, // enabled, phosphor, scanlines, curvature
    display: vec4<f32>,  // brightness, reserved...
};
@group(0) @binding(2)
var<uniform> crt: CrtUniform;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(framebuffer);
    let framebuffer_size = vec2<f32>(f32(dimensions.x), f32(dimensions.y));
    let local = position.xy - screen.viewport_origin;
    var uv = local / screen.viewport_size;
    let enabled = crt.controls.x > 0.5;

    if (enabled) {
        // Barrel distortion: remap screen coordinates before sampling.
        // Keep outside samples black instead of smearing edge pixels.
        let centered = uv * 2.0 - vec2<f32>(1.0);
        let radius2 = dot(centered, centered);
        uv = (centered * (1.0 + crt.controls.w * 0.12 * radius2) + vec2<f32>(1.0)) * 0.5;
        if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }

    let coord = clamp(
        vec2<i32>(uv * framebuffer_size),
        vec2<i32>(0),
        vec2<i32>(dimensions) - vec2<i32>(1),
    );
    let packed = textureLoad(framebuffer, coord, 0).r;
    let color = vec3<f32>(
        f32((packed >> 5u) & 0x7u) / 7.0,
        f32((packed >> 2u) & 0x7u) / 7.0,
        f32(packed & 0x3u) / 3.0,
    );
    if (!enabled) {
        return vec4<f32>(color, 1.0);
    }

    // Aperture grille: stripe pattern anchored to physical screen pixels.
    // One RGB triad every three output pixels; each stripe attenuates
    // the other two channels instead of switching them completely off.
    let stripe = u32(floor(local.x)) % 3u;
    var mask = vec3<f32>(1.0 - crt.controls.y);
    if (stripe == 0u) { mask.r = 1.0; }
    if (stripe == 1u) { mask.g = 1.0; }
    if (stripe == 2u) { mask.b = 1.0; }

    // One smooth beam profile per *logical* framebuffer row.
    // Strongest at the center, dimmer near the top/bottom of the row.
    let beam_y = fract(uv.y * framebuffer_size.y);
    let distance = abs(beam_y - 0.5) * 2.0;
    let scanline = 1.0 - crt.controls.z * smoothstep(0.25, 1.0, distance);

    let result = color * mask * scanline * crt.display.x;
    return vec4<f32>(clamp(result, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
