#version 330 core

layout(location = 0) in vec2 vertexPosition;

uniform mat4 MVP;
uniform vec4 CoordsRect;

out vec2 fragCoords;

void main() {
  fragCoords = vec2(CoordsRect.x + vertexPosition.x * CoordsRect.z,
                    CoordsRect.y + vertexPosition.y * CoordsRect.w);
  gl_Position = MVP * vec4(fragCoords, 0, 1);
}
