#version 300 es

layout(location=0) in vec2 position;
layout(location=1) in vec3 color;

layout(std140) uniform Uniforms
{
    float scale;
};

out vec2 varying_position;
out vec3 varying_color;

void main() {
    varying_position = position * scale;
    varying_color = color;

    gl_Position = vec4(varying_position, 0, 1);
}
