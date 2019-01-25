#version 330 core

in vec2 fragCoords;

out vec4 color;

const float gridThreshold = 0.05;

void main() {
  float xDist = abs(fract(fragCoords.x + 0.5) - 0.5);
  float yDist = abs(fract(fragCoords.y + 0.5) - 0.5);
  float dist = min(xDist, yDist);
  float param = 1.0 - smoothstep(0.0, gridThreshold, dist);
  color = vec4(0.0, 0.4 * param + 0.05, 0.0, 1.0);
}
