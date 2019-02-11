#version 330 core

layout(location = 0) in uvec2 vertexCorner;
layout(location = 1) in vec2 vertexUV;
layout(location = 2) in vec2 vertexRel;

uniform mat4 MVP;
uniform vec4 Rect;

out vec2 textureUV;

void main() {
  float x = Rect.x + vertexRel.x;
  if (vertexCorner.x != 0u) {
    x += Rect.z;
  }
  float y = Rect.y + vertexRel.y;
  if (vertexCorner.y != 0u) {
    y += Rect.w;
  }
  gl_Position = MVP * vec4(x, y, 0, 1);
  textureUV = vertexUV;
}
