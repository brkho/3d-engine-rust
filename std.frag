#version 150

in vec4 Color;
in vec3 Normal;
in vec3 Vert;
in vec2 TCoord;

out vec4 out_color;

uniform struct Light {
    vec3 position;
    vec3 intensity;
} light;

uniform mat4 model;
uniform mat4 normal;

void main() {
    vec3 normal_vec = normalize(mat3(normal) * Normal);
    vec3 position = vec3(model * vec4(Vert, 1.0));
    vec3 surface_to_light = normalize(light.position - position);
    float diffuse = max(dot(surface_to_light, normal_vec), 0.0);
    vec4 diffuse_color = vec4(diffuse * light.intensity * Color.rgb, Color.a);
    vec4 ambient_color = vec4(0.2 * Color.rgb, Color.a);
    out_color = clamp(diffuse_color + ambient_color, 0.0, 1.0);
}
