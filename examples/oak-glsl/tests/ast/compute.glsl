// Compute shader with a layout-only declaration and two interface blocks.
#version 450 core

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, binding = 0) buffer Output
{
    float value;
};

layout(std140, binding = 1) uniform Params
{
    vec4 tint;
};

void main()
{
    uint index = gl_GlobalInvocationID.x;
    float scaled = float(index) * 0.5f;
    if (index >= 64u)
    {
        return;
    }
    value = scaled * tint.x;
}
