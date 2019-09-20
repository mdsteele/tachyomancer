#version 330 core

layout(location = 0) in uvec2 vertexUV;

uniform mat4 MVP;
uniform uint PortraitIndex;

out vec2 textureUV;

void main() {
  gl_Position = MVP * vec4(vertexUV.x * 68u, vertexUV.y * 85u, 0, 1);
  uint col = (PortraitIndex % 3u) + vertexUV.x;
  uint row = (PortraitIndex / 3u) + vertexUV.y;
  float u = 0.265625 * col;
  float v = 0.33203125 * row;
  textureUV = vec2(u, v);
}
