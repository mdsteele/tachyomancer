#version 330 core

in vec2 textureUV;

uniform sampler2D Font;

out vec3 color;

void main() {
  float gray = texture(Font, textureUV).r;
  color = vec3(gray, gray, gray);
}
