// Fill a storage buffer with the invocation index.
@group(0) @binding(0) var<storage, read_write> out: array<f32>;

@compute @workgroup_size(64, 1, 1)
fn fill(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.x;
    out[index] = f32(index);
}
