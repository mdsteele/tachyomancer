#version 330 core

in vec2 textureUV;

uniform vec4 Color;
uniform sampler2D Font;

out vec4 color;

void main() {
  color = Color;
  color.a *= texture(Font, textureUV).r;
}
