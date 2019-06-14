#version 330 core

in vec2 textureUV;

uniform vec4 IconColor;
uniform sampler2D IconTexture;

out vec4 color;

void main() {
  float alpha = texture(IconTexture, textureUV).r;
  color = vec4(IconColor.rgb, IconColor.a * alpha);
}
