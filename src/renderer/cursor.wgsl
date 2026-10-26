
@group(0) @binding(0)
var cursor_texture: texture_2d<f32>;

@group(0) @binding(1)
var cursor_sampler: sampler;

struct CursorUniform {
    position: vec2<f32>,
    size: vec2<f32>,
    screen_size: vec2<f32>,
    padding: vec2<f32>,
};

@group(0) @binding(2)
var<uniform> cursor: CursorUniform;

struct ScreenUniform {
    viewport_origin: vec2<f32>,
    viewport_size: vec2<f32>,
};

@group(0) @binding(3)
var<uniform> screen: ScreenUniform;

struct CrtUniform {
    controls: vec4<f32>,
    display: vec4<f32>,
};

@group(0) @binding(4)
var<uniform> crt: CrtUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

// Full-window triangle. The cursor's shape is determined
// by texture sampling in the fragment shader.
@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(
        positions[vertex_index], 0.0, 1.0
    );

    return output;
}

// Same physical-screen -> framebuffer UV mapping
// used by framebuffer.wgsl.
fn distort_uv(uv: vec2<f32>) -> vec2<f32> {
    let centered = uv * 2.0 - vec2<f32>(1.0);
    let radius2 = dot(centered, centered);
    let factor = 1.0 + crt.controls.w * 0.12 * radius2;

    return (centered * factor + vec2<f32>(1.0)) * 0.5;
}

fn inside_unit(uv: vec2<f32>) -> bool {
    return all(uv >= vec2<f32>(0.0))
        && all(uv < vec2<f32>(1.0));
}

@fragment
fn fs_main(
    input: VertexOutput
) -> @location(0) vec4<f32> {
    let pixel = input.position.xy;
    let enabled = crt.controls.x > 0.5;

    // Original cursor coordinates in physical window pixels.
    // cursor.position is the mouse hotspot.
    let normal_cursor_uv =
        (pixel - cursor.position) / cursor.size;

    // The physical location of this fragment relative
    // to the CRT viewport.
    let screen_uv =
        (pixel - screen.viewport_origin)
        / screen.viewport_size;

    let inside_viewport = inside_unit(screen_uv);

    var crt_uv = screen_uv;
    if (enabled && inside_viewport) {
        crt_uv = distort_uv(screen_uv);
    }

    let inside_crt =
        inside_viewport && inside_unit(crt_uv);

    // Outside the CRT, render the cursor normally.
    if (!enabled || !inside_crt) {
        if (!inside_unit(normal_cursor_uv)) {
            return vec4<f32>(0.0);
        }

        return textureSampleLevel(
            cursor_texture,
            cursor_sampler,
            normal_cursor_uv,
            0.0
        );
    }

    // Find the framebuffer-space position corresponding
    // to the physical mouse hotspot.
    let mouse_screen_uv =
        (cursor.position - screen.viewport_origin)
        / screen.viewport_size;

    // Only anchor the distorted cursor to the CRT
    // mapping when its hotspot is inside the CRT.
    var mouse_crt_uv = mouse_screen_uv;

    if (inside_unit(mouse_screen_uv)) {
        mouse_crt_uv = distort_uv(mouse_screen_uv);
    }

    // Convert the cursor's physical dimensions to
    // normalized viewport dimensions.
    let cursor_size_uv = cursor.size / screen.viewport_size;

    // Sample the cursor in the same distorted coordinate
    // space as the underlying framebuffer.
    let distorted_cursor_uv =
        (crt_uv - mouse_crt_uv) / cursor_size_uv;

    if (!inside_unit(distorted_cursor_uv)) {
        return vec4<f32>(0.0);
    }

    let sampled = textureSampleLevel(
        cursor_texture,
        cursor_sampler,
        distorted_cursor_uv,
        0.0
    );

    if (sampled.a <= 0.0) {
        return vec4<f32>(0.0);
    }

    // Aperture grille, matching framebuffer.wgsl.
    let local = pixel - screen.viewport_origin;
    let stripe = u32(floor(local.x)) % 3u;

    var mask = vec3<f32>(1.0 - crt.controls.y);

    if (stripe == 0u) { mask.r = 1.0; }
    if (stripe == 1u) { mask.g = 1.0; }
    if (stripe == 2u) { mask.b = 1.0; }

    // Scanlines aligned to logical framebuffer rows.
    // This uses the established 480x270 framebuffer.
    let framebuffer_height = 270.0;

    let beam_y = fract(crt_uv.y * framebuffer_height);
    let distance = abs(beam_y - 0.5) * 2.0;

    let scanline = 1.0 - crt.controls.z
        * smoothstep(0.25, 1.0, distance);

    let color = sampled.rgb
        * mask
        * scanline
        * crt.display.x;

    return vec4<f32>(
        clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)),
        sampled.a
    );
}
