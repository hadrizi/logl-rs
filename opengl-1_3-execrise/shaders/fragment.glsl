#version 330 core
in vec3 ourColor;
in vec3 fPos;

out vec4 FragColor;

void main() {
    FragColor = vec4(fPos, 1.0);
}