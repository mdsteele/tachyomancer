#version 330 core

in vec2 textureUV;

uniform vec3 Color;
uniform sampler2D Font;

out vec4 color;

void main() {
  float alpha = texture(Font, textureUV).r;
  color = vec4(Color, alpha);
}
