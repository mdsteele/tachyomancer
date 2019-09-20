#version 330 core

in vec2 textureUV;

uniform sampler2D Texture;

out vec4 color;

void main() {
  color = mix(vec4(0.118, 0.039, 0.180, 1.0),
              vec4(0.592, 0.949, 0.988, 1.0),
              texture(Texture, textureUV).r);
}
