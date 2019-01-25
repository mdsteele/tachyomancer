#version 330 core

in vec2 textureUV;

uniform sampler2D IconTexture;

out vec4 color;

void main() {
  float alpha = texture(IconTexture, textureUV).r;
  color = vec4(0.3 + 0.5 * alpha, 0.3 + 0.25 * alpha, 0.3, 1.0);
}
