#version 150

in vec2 Texcoord;

out vec4 out_color;

uniform sampler2D brian_tex;
uniform sampler2D samantha_tex;
uniform float elapsed;

void main() {
    // vec4 brian_col = texture(brian_tex, Texcoord);
    // vec4 samantha_col = texture(samantha_tex, Texcoord);
    // out_color = mix(brian_col, samantha_col, (sin(elapsed) + 1) / 2.0);
    vec2 coord;
    if (Texcoord.y < 0.5) {
      coord = vec2(Texcoord.x, Texcoord.y);
      out_color = texture(samantha_tex, coord);
    } else {
      coord = vec2(Texcoord.x + sin(Texcoord.y * 25 + elapsed) / 20, (1 - Texcoord.y));
      out_color = texture(samantha_tex, coord) * vec4(0.6, 0.6, 0.6, 1.0);
    }
}
