#version 330

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

out vec3 v_color;

uniform float screen_ratio;

void main() {
    gl_Position = vec4((position.x * screen_ratio) / 30.0, (position.y) / 30.0, 1.0, 1.0);
    v_color = color;
}
