#version 150

in vec2 Texcoord;

out vec4 out_color;

uniform sampler2D brian_tex;
uniform sampler2D samantha_tex;

void main() {
    vec4 brian_col = texture(brian_tex, Texcoord);
    vec4 samantha_col = texture(samantha_tex, Texcoord);
    out_color = mix(brian_col, samantha_col, 0.5);
}
