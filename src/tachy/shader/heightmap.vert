#version 330 core

layout(location = 0) in vec2 vertexUV;

uniform mat4 MV;
uniform mat4 P;
uniform sampler2D Heightmap;

out VS_OUT {
  vec3 normalCamSpace;
  vec2 textureUV;
} vs_out;

const float epsilon = 1.0 / 128.0;

vec3 modelPos(vec2 uv) {
  return vec3(uv.x, texture(Heightmap, uv).r, uv.y);
}

void main() {
  vs_out.textureUV = vertexUV;

  vec3 north = modelPos(vec2(vertexUV.x, vertexUV.y - epsilon));
  vec3 south = modelPos(vec2(vertexUV.x, vertexUV.y + epsilon));
  vec3 west = modelPos(vec2(vertexUV.x - epsilon, vertexUV.y));
  vec3 east = modelPos(vec2(vertexUV.x + epsilon, vertexUV.y));
  vec3 normalModelSpace = cross(south - north, east - west);
  vs_out.normalCamSpace = normalize((MV * vec4(normalModelSpace, 0)).xyz);

  vec3 positionModelSpace = modelPos(vertexUV);
  vec3 positionCamSpace = (MV * vec4(positionModelSpace, 1)).xyz;
  gl_Position = P * vec4(positionCamSpace, 1);
}
