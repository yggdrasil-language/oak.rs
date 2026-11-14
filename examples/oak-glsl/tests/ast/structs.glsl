// Struct, layout qualifier, and uniform coverage.
#version 450 core

struct Light
{
    vec3 position;
    float intensity;
};

layout(location = 0) in vec3 vNormal;
layout(location = 0) out vec4 fragColor;

uniform mat4 uModel;
uniform sampler2D uTexture;

const float PI = 3.14159;

vec3 shade(Light light, vec3 normal)
{
    vec3 direction = normalize(light.position);
    float diffuse = max(dot(normal, direction), 0.0);
    return light.intensity * diffuse * vec3(0.5, 0.5, 0.5);
}

void main()
{
    Light light;
    light.position = vec3(0.0, 1.0, 0.0);
    light.intensity = 1.0;
    vec3 color = shade(light, vNormal);
    fragColor = texture(uTexture, color.xy);
    if (color.x < 0.0)
    {
        discard;
    }
}
