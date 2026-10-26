@group(0) @binding(0)
var frame_texture: texture_2d<f32>;

@group(0) @binding(1)
var frame_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );

    var output: VertexOutput;
    let pos = positions[vertex_index];

    output.position = vec4<f32>(pos, 0.0, 1.0);
    output.uv = vec2<f32>(
        (pos.x + 1.0) * 0.5,
        (1.0 - pos.y) * 0.5,
    );

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(frame_texture, frame_sampler, input.uv);
}