#version 330 core

layout(location = 0) in vec3 positionModelSpace;
layout(location = 1) in vec3 normalModelSpace;
layout(location = 2) in vec3 materialColor;
layout(location = 3) in vec2 textureUV;

uniform mat4 MV;
uniform mat4 P;

out VS_OUT {
  vec3 normalCamSpace;
  vec3 materialColor;
  vec2 textureUV;
} vs_out;

void main() {
  vec3 positionCamSpace = (MV * vec4(positionModelSpace, 1)).xyz;
  vs_out.normalCamSpace = normalize((MV * vec4(normalModelSpace, 0)).xyz);
  vs_out.materialColor = materialColor;
  vs_out.textureUV = textureUV;
  gl_Position = P * vec4(positionCamSpace, 1);
}
