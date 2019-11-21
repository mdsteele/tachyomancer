#version 330 core

layout(location = 0) in uvec2 vertex;

uniform vec4 TexRect;
uniform mat4 MVP;

out vec2 textureUV;

void main() {
  textureUV = vec2(TexRect.x + vertex.x * TexRect.z,
                   TexRect.y + vertex.y * TexRect.w);
  gl_Position = MVP * vec4(vertex, 0, 1);
}
