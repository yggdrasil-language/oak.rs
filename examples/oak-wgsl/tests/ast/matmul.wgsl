// Naive matrix multiplication, one invocation per output element.
@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;

const N: u32 = 4u;

fn index(row: u32, col: u32) -> u32 {
    return row * N + col;
}

@compute @workgroup_size(1, 1, 1)
fn matmul(@builtin(global_invocation_id) gid: vec3<u32>) {
    let row = gid.y;
    let col = gid.x;
    var sum = 0.0;
    var scaled = 1.0 + 2.0 * 3.0;
    for (var k = 0u; k < N; k = k + 1u) {
        sum = sum + a[index(row, k)] * b[index(k, col)];
    }
    if (sum > 100.0) {
        sum = 100.0;
    }
    result[index(row, col)] = sum;
}
