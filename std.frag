#version 150

#define MAX_LIGHTS 8
#define EMPTY_LIGHT 0u
#define POINT_LIGHT 1u
#define DIRECTIONAL_LIGHT 2u
#define SPOT_LIGHT 3u

// TODO: Add materials so these can be a uniforms.
#define AMBIENT_COEFF 0.2
#define SPECULAR_COLOR 1.0

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

uniform vec3 camera;
uniform vec4 color;
uniform mat4 model;
uniform mat4 normal;
uniform float specular_coeff;
uniform sampler2D diffuse_map;
uniform sampler2D specular_map;
uniform sampler2D normal_map;

void main() {
    // Ambient light.
    vec4 total_color = vec4(AMBIENT_COEFF * color.rgb, 0.0);
    Light light = lights[7];
    if (light.type != EMPTY_LIGHT) {
        vec3 position = vec3(model * vec4(Vert, 1.0));
        vec3 norm = normalize(mat3(normal) * ((texture(normal_map, TCoord).rgb - 0.5) * 2));
        // vec3 norm = normalize(mat3(normal) * Normal);
        vec3 surface_to_light;
        vec3 intensity;

        if (light.type == DIRECTIONAL_LIGHT) {
            surface_to_light = -normalize(light.direction);
            intensity = light.intensity;
        } else {
            // Point light.
            surface_to_light = normalize(light.position - position);
            float dist = distance(position, light.position);
            intensity = light.intensity /
                (light.const_attn + light.linear_attn * dist + light.quad_attn * (dist * dist));
            if (light.type == SPOT_LIGHT) {
                // bla = 1.0;
                float cos_dv = dot(normalize(light.direction), -surface_to_light);
                if (cos_dv > cos(light.cutoff)) {
                    intensity *= pow(cos_dv, light.dropoff);
                } else {
                    intensity = vec3(0, 0, 0);
                }
            }
        }

        // Get diffuse lighting.
        float cos_nl = max(dot(surface_to_light, norm), 0.0);
        vec4 diffuse = vec4(cos_nl * intensity * color.rgb *
                texture(diffuse_map, TCoord).rgb, color.a);

        // Get specular lighting.
        vec4 specular = vec4(0, 0, 0, 0);
        if (cos_nl > 0.0) {
            vec3 surface_to_camera = normalize(camera - position);
            vec3 halfway = normalize(surface_to_light + surface_to_camera);
            float cos_nha = pow(max(dot(norm, halfway), 0.0), specular_coeff);
            specular = vec4(cos_nha * texture(specular_map, TCoord).rgb * intensity, 0.0);
        }

        // Get ambient lighting.
        total_color += diffuse + specular;
    }
    light = lights[6];
    if (light.type != EMPTY_LIGHT) {
        vec3 position = vec3(model * vec4(Vert, 1.0));
        vec3 norm = normalize(mat3(normal) * ((texture(normal_map, TCoord).rgb - 0.5) * 2));
        // vec3 norm = normalize(mat3(normal) * Normal);
        vec3 surface_to_light;
        vec3 intensity;

        if (light.type == DIRECTIONAL_LIGHT) {
            surface_to_light = -normalize(light.direction);
            intensity = light.intensity;
        } else {
            // Point light.
            surface_to_light = normalize(light.position - position);
            float dist = distance(position, light.position);
            intensity = light.intensity /
                (light.const_attn + light.linear_attn * dist + light.quad_attn * (dist * dist));
            if (light.type == SPOT_LIGHT) {
                // bla = 1.0;
                float cos_dv = dot(normalize(light.direction), -surface_to_light);
                if (cos_dv > cos(light.cutoff)) {
                    intensity *= pow(cos_dv, light.dropoff);
                } else {
                    intensity = vec3(0, 0, 0);
                }
            }
        }

        // Get diffuse lighting.
        float cos_nl = max(dot(surface_to_light, norm), 0.0);
        vec4 diffuse = vec4(cos_nl * intensity * color.rgb *
                texture(diffuse_map, TCoord).rgb, color.a);

        // Get specular lighting.
        vec4 specular = vec4(0, 0, 0, 0);
        if (cos_nl > 0.0) {
            vec3 surface_to_camera = normalize(camera - position);
            vec3 halfway = normalize(surface_to_light + surface_to_camera);
            float cos_nha = pow(max(dot(norm, halfway), 0.0), specular_coeff);
            specular = vec4(cos_nha * texture(specular_map, TCoord).rgb * intensity, 0.0);
        }

        // Get ambient lighting.
        total_color += diffuse + specular;
    }
    light = lights[5];
    if (light.type != EMPTY_LIGHT) {
        vec3 position = vec3(model * vec4(Vert, 1.0));
        vec3 norm = normalize(mat3(normal) * ((texture(normal_map, TCoord).rgb - 0.5) * 2));
        // vec3 norm = normalize(mat3(normal) * Normal);
        vec3 surface_to_light;
        vec3 intensity;

        if (light.type == DIRECTIONAL_LIGHT) {
            surface_to_light = -normalize(light.direction);
            intensity = light.intensity;
        } else {
            // Point light.
            surface_to_light = normalize(light.position - position);
            float dist = distance(position, light.position);
            intensity = light.intensity /
                (light.const_attn + light.linear_attn * dist + light.quad_attn * (dist * dist));
            if (light.type == SPOT_LIGHT) {
                // bla = 1.0;
                float cos_dv = dot(normalize(light.direction), -surface_to_light);
                if (cos_dv > cos(light.cutoff)) {
                    intensity *= pow(cos_dv, light.dropoff);
                } else {
                    intensity = vec3(0, 0, 0);
                }
            }
        }

        // Get diffuse lighting.
        float cos_nl = max(dot(surface_to_light, norm), 0.0);
        vec4 diffuse = vec4(cos_nl * intensity * color.rgb *
                texture(diffuse_map, TCoord).rgb, color.a);

        // Get specular lighting.
        vec4 specular = vec4(0, 0, 0, 0);
        if (cos_nl > 0.0) {
            vec3 surface_to_camera = normalize(camera - position);
            vec3 halfway = normalize(surface_to_light + surface_to_camera);
            float cos_nha = pow(max(dot(norm, halfway), 0.0), specular_coeff);
            specular = vec4(cos_nha * texture(specular_map, TCoord).rgb * intensity, 0.0);
        }

        // Get ambient lighting.
        total_color += diffuse + specular;
    }

    out_color = clamp(total_color, 0.0, 1.0);
}
