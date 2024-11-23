// Compute kernel that fills an output buffer.
RWStructuredBuffer<float4> output : register(u0);

cbuffer Params : register(b0)
{
    float4 tint;
    float exposure;
};

[numthreads(64, 1, 1)]
void cs_main(uint3 id : SV_DispatchThreadID)
{
    uint index = id.x;
    float4 value = float4(1.0f, 0.5f, 0.25f, 1.0f);
    if (index >= 64u)
    {
        return;
    }
    output[index] = value * tint;
}
