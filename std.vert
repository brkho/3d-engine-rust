#version 150

in vec2 position;
in float color;

out float Color;

void main() {
    Color = color;
    gl_Position = vec4(position.x, position.y, 0.0, 1.0);
}
