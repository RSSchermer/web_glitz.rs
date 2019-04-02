#version 300 es

layout(location=0) in vec2 position;
layout(location=1) in vec3 color;

out vec3 varying_color;

void main() {
    varying_color = color;

    gl_Position = vec4(position, 0, 1);
}
