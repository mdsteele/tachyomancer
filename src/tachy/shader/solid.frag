#version 330 core

uniform vec3 SolidColor;

out vec4 color;

void main() {
  color = vec4(SolidColor, 1.0);
}
