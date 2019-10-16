#version 330 core

layout(location = 0) in vec3 vertexPos;
layout(location = 1) in vec2 vertexUV;
layout(location = 2) in uvec2 vertexCorner;

uniform mat4 MVP;
uniform vec2 ChipSize;
uniform vec4 TexRect;

out VS_OUT {
  vec2 textureUV;
} vs_out;

void main() {
  float x = vertexPos.x;
  if (vertexCorner.x == 0u) {
    x -= 0.5 * ChipSize.x;
  } else {
    x += 0.5 * ChipSize.x;
  }
  float y = vertexPos.y;
  if (vertexCorner.y == 0u) {
    y -= 0.5 * ChipSize.y;
  } else {
    y += 0.5 * ChipSize.y;
  }
  gl_Position = MVP * vec4(x, y, vertexPos.z, 1);
  vs_out.textureUV = vec2(TexRect.x + vertexUV.x * TexRect.z,
                          TexRect.y + vertexUV.y * TexRect.w);
}
