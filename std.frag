#version 150

#define MAX_LIGHTS 16
#define EMPTY_LIGHT 0
#define POINT_LIGHT 1
#define DIRECTIONAL_LIGHT 2
#define SPOT_LIGHT 3

in vec4 Color;
in vec3 Normal;
in vec3 Vert;
in vec2 TCoord;

out vec4 out_color;

uniform struct Light {
    uint type;
    vec3 intensity;
    vec3 position;
    vec3 direction;
    float const_attn;
    float linear_attn;
    float quad_attn;
    float cutoff;
    float dropoff;
} lights[MAX_LIGHTS];

uniform mat4 model;
uniform mat4 normal;

void main() {
    Light light = lights[15];
    vec3 position = vec3(model * vec4(Vert, 1.0));
    float dist = distance(position, light.position);
    vec3 norm = normalize(mat3(normal) * Normal);
    vec3 surface_to_light = normalize(light.position - position);
    float cos_nl = max(dot(surface_to_light, norm), 0.0);
    vec3 intensity = light.intensity /
            (light.const_attn + light.linear_attn * dist + light.quad_attn * (dist * dist));

    vec4 diffuse = vec4(cos_nl * intensity * Color.rgb, Color.a);
    vec4 ambient_color = vec4(0.2 * Color.rgb, Color.a);
    out_color = clamp(diffuse + ambient_color, 0.0, 1.0);
}
