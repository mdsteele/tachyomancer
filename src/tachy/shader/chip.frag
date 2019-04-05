#version 330 core

in vec2 textureUV;

uniform vec3 IconColor;
uniform sampler2D IconTexture;

out vec4 color;

void main() {
  float alpha = texture(IconTexture, textureUV).r;
  color = vec4(vec3(0.3, 0.3, 0.3) + IconColor * (0.7 * alpha), 1.0);
}
