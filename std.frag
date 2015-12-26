#version 150

in float Color;

out vec4 out_color;

void main() {
    out_color = vec4(Color, Color, Color, 1.0);
}
