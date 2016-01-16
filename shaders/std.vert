#version 150

in vec3 position;
in vec3 normal;
in vec3 tangent;
in vec3 bitangent;
in vec2 tcoord;

out vec3 Normal;
out mat3 TBN;
out vec3 Vert;
out vec2 TCoord;

uniform mat4 normal_matrix;
uniform mat4 transform;

void main() {
    // TODO: Orthognalize TBN.
    Normal = normal;
    vec3 T = normalize(vec3(normal_matrix * vec4(tangent, 0.0)));
    vec3 B = normalize(vec3(normal_matrix * vec4(bitangent, 0.0)));
    vec3 N = normalize(vec3(normal_matrix * vec4(normal, 0.0)));
    TBN = mat3(T, B, N);
    TCoord = tcoord;
    Vert = position;
    gl_Position = transform * vec4(position, 1.0);
}
