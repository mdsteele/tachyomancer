#version 330 core

in vec2 textureUV;

uniform sampler2D Texture;
uniform vec4 ColorTint;

out vec4 color;

void main() {
  color = texture(Texture, textureUV) * ColorTint;
}
