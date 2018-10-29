#version 330 core

layout(location = 0) in uvec2 vertexUV;
layout(location = 1) in uint textIndex;

uniform mat4 MVP;
uniform uint Text[64];

out vec2 textureUV;

void main() {
  float x = float(vertexUV.x + textIndex);
  float y = float(vertexUV.y);
  gl_Position = MVP * vec4(x, y, 0, 1);

  uint chr = Text[textIndex];
  float u = float((chr % 16u) + vertexUV.x) / 16.0;
  float v = float((chr / 16u) + vertexUV.y) / 16.0;
  textureUV = vec2(u, v);
}
