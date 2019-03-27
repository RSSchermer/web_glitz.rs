#version 300 es

in vec2 position;
in vec3 color;

out vec3 varying_color;

void main() {
    varying_color = color;

    gl_Position = vec4(position, 0, 1);
}
