#version 330 core

in vec2 textureUV;

uniform vec4 IconColor;
uniform sampler2D IconTexture;

out vec4 color;

void main() {
  color = IconColor * texture(IconTexture, textureUV).r;
}
