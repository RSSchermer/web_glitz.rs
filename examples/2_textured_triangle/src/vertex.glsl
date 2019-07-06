#version 300 es

layout(location=0) in vec2 position;
layout(location=1) in vec2 texture_coordinates;

out vec2 varying_texture_coordinates;

void main() {
    varying_texture_coordinates = texture_coordinates;

    gl_Position = vec4(position, 0, 1);
}
