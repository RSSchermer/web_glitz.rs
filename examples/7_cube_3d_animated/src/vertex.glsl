#version 300 es

layout(location=0) in vec4 position;
layout(location=1) in vec3 color;

layout(std140) uniform Uniforms
{
    mat4 model;
    mat4 view;
    mat4 projection;
};

out vec3 varying_color;

void main() {
    varying_color = color;

    gl_Position = projection * view * model * position;
}
