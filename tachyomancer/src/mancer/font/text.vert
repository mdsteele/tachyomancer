#version 330 core

layout(location = 0) in uvec2 vertexUV;
layout(location = 1) in uint textIndex;

uniform mat4 MVP;
uniform uint Text[64];
uniform float Slant;
uniform vec2 CharTexSize;

out vec2 textureUV;

void main() {
  float x = float(vertexUV.x) + float(textIndex);
  float y = float(vertexUV.y);
  if (vertexUV.y == 0u) {
    x += Slant;
  } else {
    x -= Slant;
  }
  gl_Position = MVP * vec4(x, y, 0, 1);

  uint chr = Text[textIndex];
  float u = float((chr % 16u) + vertexUV.x) * CharTexSize.x;
  float v = float((chr / 16u) + vertexUV.y) * CharTexSize.y;
  textureUV = vec2(u, v);
}
