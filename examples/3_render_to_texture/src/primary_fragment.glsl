#version 300 es
precision mediump float;

in vec2 varying_texture_coordinates;

out vec4 out_color;

uniform sampler2D diffuse_texture;

void main() {
    out_color = texture(diffuse_texture, varying_texture_coordinates);
}
