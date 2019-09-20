#version 330 core

layout(location = 0) in uvec2 vertexPos;

uniform mat4 MVP;
uniform vec2 FrameSize;
uniform vec2 TexSize;

out vec2 textureUV;

void main() {
  gl_Position = MVP * vec4(vertexPos.x * FrameSize.x,
                           vertexPos.y * FrameSize.y, 0, 1);
  textureUV = vec2(vertexPos.x * TexSize.x, vertexPos.y * TexSize.y);
}
