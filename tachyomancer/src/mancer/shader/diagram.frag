#version 330 core

in vec2 textureUV;

uniform vec4 ColorMult;
uniform sampler2D Texture;

out vec4 color;

void main() {
  color = texture(Texture, textureUV) * ColorMult;
}
