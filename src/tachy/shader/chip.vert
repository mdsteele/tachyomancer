#version 330 core

layout(location = 0) in vec3 vertexPos;
layout(location = 1) in vec2 vertexUV;

uniform mat4 MVP;
uniform vec4 TexRect;

out VS_OUT {
  vec2 textureUV;
} vs_out;

void main() {
  gl_Position = MVP * vec4(vertexPos, 1);
  vs_out.textureUV = vec2(TexRect.x + vertexUV.x * TexRect.z,
                          TexRect.y + vertexUV.y * TexRect.w);
}
