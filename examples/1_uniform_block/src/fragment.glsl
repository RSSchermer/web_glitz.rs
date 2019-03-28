#version 300 es
precision mediump float;

in vec3 varying_color;

out vec4 out_color;

void main() {
    out_color = vec4(varying_color, 1);
}
