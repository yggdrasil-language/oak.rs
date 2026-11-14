// Minimal vertex and fragment pair sharing a struct.
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.color = vec3<f32>(1.0, 0.5, 0.25);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
