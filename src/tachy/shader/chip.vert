#version 330 core

layout(location = 0) in uvec2 vertexUV;

uniform mat4 MVP;
uniform uvec2 IconCoords;

out vec2 textureUV;

void main() {
  gl_Position = MVP * vec4(vertexUV, 0, 1);

  float u = float(IconCoords.x + vertexUV.x) / 8.0;
  float v = float(IconCoords.y + vertexUV.y) / 8.0;
  textureUV = vec2(u, v);
}
