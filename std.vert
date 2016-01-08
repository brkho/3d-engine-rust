#version 150

in vec3 position;
in vec3 normal;
in vec2 tcoord;
in vec4 color;

out vec4 Color;

uniform mat4 transform;

void main() {
    Color = color;
    gl_Position = transform * vec4(position.x, position.y, position.z, 1.0);
}
