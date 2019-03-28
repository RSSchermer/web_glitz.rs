#version 300 es

in vec2 position;
in vec3 color;

layout(std140) uniform uniforms
{
    float scale;
};

out vec3 varying_color;

void main() {
    varying_color = color;

    gl_Position = vec4(position * scale, 0, 1);
}
