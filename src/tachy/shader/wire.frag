#version 330 core

in float fragment_tex_param;

uniform vec4 WireColor;
uniform sampler1D WireTexture;

out vec4 color;

void main() {
  color = vec4(WireColor.rgb * texture(WireTexture, fragment_tex_param).r,
               WireColor.a);
}
