#version 150

in vec2 Texcoord;

out vec4 out_color;

uniform sampler2D brian_tex;
uniform sampler2D samantha_tex;
uniform float elapsed;

void main() {
    vec2 coord;
    if (Texcoord.y < 0.5) {
      coord = vec2(Texcoord.x, Texcoord.y * 2);
      out_color = texture(samantha_tex, coord);
    } else {
      coord = vec2(Texcoord.x + sin(Texcoord.y * 25 + elapsed) / 20, (1 - Texcoord.y) * 2);
      out_color = texture(samantha_tex, coord) * vec4(0.6, 0.6, 0.6, 1.0);
    }
}
