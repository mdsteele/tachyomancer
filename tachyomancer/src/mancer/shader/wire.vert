#version 330 core

layout(location = 0) in vec2 vertexPosition_modelspace;
layout(location = 1) in float vertex_tex_param;

uniform mat4 MVP;

out float fragment_tex_param;

void main() {
  gl_Position = MVP * vec4(vertexPosition_modelspace, 0.025, 1);
  fragment_tex_param = vertex_tex_param;
}
