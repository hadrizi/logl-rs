#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aCol;

out vec3 fPos;
out vec3 ourColor;

uniform float offset;

void main() {
    gl_Position = vec4(aPos, 1.0);
    gl_Position.x += offset;
    ourColor = aCol;
    fPos = aPos;
}