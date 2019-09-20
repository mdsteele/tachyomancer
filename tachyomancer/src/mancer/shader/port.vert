#version 330 core

layout(location = 0) in vec2 vertexXY;
layout(location = 1) in uint vertexEdge;

uniform mat4 MVP;
uniform uint FlowAndColor;
uniform float WidthScale;

out vec2 textureUV;

const uint portFlowMask = 2u;
const uint portColorMask = 1u;

void main() {
  float x = vertexXY.x; // 0 to 1
  float y = vertexXY.y; // -1 to 1
  if (vertexEdge != 0u) {
    float delta = 0.0;
    if ((FlowAndColor & portColorMask) != 0u) {
      delta = 1.1 * (abs(2 * y) - 1); // event
    } else {
      delta = 0.8 * (pow(1.414 * y, 2) - 1); // behavior
    }
    if ((FlowAndColor & portFlowMask) != 0u) {
      delta = -delta;
    }
    x += 0.06 * delta;
  }
  gl_Position = MVP * vec4(x, y * WidthScale, 0, 1);

  float u = 0.5 * (y + 1);
  float v = x;
  textureUV = vec2(u, v);
}
