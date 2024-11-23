// Struct, qualifier, semantic, register, and alias coverage.
struct VSInput
{
    float4 position : POSITION;
    float2 texcoord : TEXCOORD0;
    float3 normal : NORMAL;
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD0;
};

using Color = float4;
typedef float3 Vector3;

uniform float4x4 gWorld;
Texture2D gTexture : register(t0);
SamplerState gSampler : register(s0);

VSOutput vs_main(VSInput input)
{
    VSOutput output;
    float4 world = mul(input.position, gWorld);
    output.position = world;
    output.texcoord = input.texcoord;
    return output;
}
