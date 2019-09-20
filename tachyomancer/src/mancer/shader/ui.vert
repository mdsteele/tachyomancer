#version 330 core

layout(location = 0) in uvec2 vertexCorner;
layout(location = 1) in vec2 vertexUV;
layout(location = 2) in vec2 vertexRel;

uniform mat4 MVP;
uniform vec4 ScreenRect;
uniform vec4 TexRect;

out vec2 textureUV;

void main() {
  float x = ScreenRect.x + vertexRel.x;
  if (vertexCorner.x != 0u) {
    x += ScreenRect.z;
  }
  float y = ScreenRect.y + vertexRel.y;
  if (vertexCorner.y != 0u) {
    y += ScreenRect.w;
  }
  gl_Position = MVP * vec4(x, y, 0, 1);
  textureUV = vec2(TexRect.x + vertexUV.x * TexRect.z,
                   TexRect.y + vertexUV.y * TexRect.w);
}
