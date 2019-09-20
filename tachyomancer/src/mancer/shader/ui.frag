#version 330 core

in vec2 textureUV;

uniform sampler2D Texture;
uniform vec4 Color1;
uniform vec4 Color2;
uniform vec4 Color3;

out vec4 color;

void main() {
  vec4 texColor = texture(Texture, textureUV);
  color = Color1 * texColor.r + Color2 * texColor.g + Color3 * texColor.b;
  color.a *= texColor.a;
}
