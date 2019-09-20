#version 330 core

layout(location = 0) in vec2 vertexPosition;

uniform mat4 MVP;
uniform vec4 CoordsRect;

out vec2 fragCoords;

void main() {
  gl_Position = MVP * vec4(vertexPosition, 0, 1);
  fragCoords = vec2(CoordsRect.x + vertexPosition.x * CoordsRect.z,
                    CoordsRect.y + vertexPosition.y * CoordsRect.w);
}
