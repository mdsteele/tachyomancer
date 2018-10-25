#version 330 core

in float fragment_tex_param;

uniform vec3 WireColor;
uniform sampler1D WireTexture;

out vec3 color;

void main() {
  color = texture(WireTexture, fragment_tex_param).r * WireColor;
}
