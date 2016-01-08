#version 150

in vec3 position;
in vec3 normal;
in vec2 tcoord;
in vec4 color;

out vec4 Color;
out vec3 Normal;
out vec3 Vert;
out vec2 TCoord;

uniform mat4 transform;

void main() {
    Color = color;
    Normal = normal;
    TCoord = tcoord;
    Vert = position;
    gl_Position = transform * vec4(position, 1.0);
}
