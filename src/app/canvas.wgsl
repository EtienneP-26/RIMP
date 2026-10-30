struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// One big triangle that covers the whole screen, built from the vertex index alone.
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);

    var out: VertexOutput;
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

@group(0) @binding(0) var canvas_texture: texture_2d<f32>;
@group(0) @binding(1) var canvas_sampler: sampler;
// Maps a screen uv to a canvas uv (inverse of the viewport). Columns are padded to 16 bytes.
@group(0) @binding(2) var<uniform> view_to_canvas: mat3x3<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (view_to_canvas * vec3<f32>(in.uv, 1.0)).xy;
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        discard;
    }
    return textureSample(canvas_texture, canvas_sampler, uv);
}
