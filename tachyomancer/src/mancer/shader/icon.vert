#version 330 core

layout(location = 0) in uvec2 vertex;

uniform uint IconIndex;
uniform mat4 MVP;

out vec2 textureUV;

void main() {
  float col = (IconIndex % 4u) + vertex.x;
  float row = (IconIndex / 4u) + vertex.y;
  textureUV = vec2(0.25 * col, 0.25 * row);
  gl_Position = MVP * vec4(vertex, 0, 1);
}
