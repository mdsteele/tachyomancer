#version 330 core

layout(location = 0) in vec2 vertexPosition_modelspace;

uniform mat4 MVP;
uniform vec2 RectSize;
uniform float ShadowRadius;

out vec2 relativePoint;

void main() {
  gl_Position = MVP * vec4(vertexPosition_modelspace, 0, 1);
  float relX = vertexPosition_modelspace.x * (RectSize.x + ShadowRadius * 2) -
    ShadowRadius;
  float relY = vertexPosition_modelspace.y * (RectSize.y + ShadowRadius * 2) -
    ShadowRadius;
  relativePoint = vec2(relX, relY);
}
