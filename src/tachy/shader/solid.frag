#version 330 core

in float fragment_tex_param; // FIXME

uniform vec3 SolidColor;

out vec3 color;

void main() {
  color = SolidColor;
}
