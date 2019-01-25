#version 330 core

in vec2 textureUV;

uniform sampler2D Font;
uniform vec3 TextColor;

out vec4 color;

void main() {
  float alpha = texture(Font, textureUV).r;
  color = vec4(TextColor, alpha);
}
