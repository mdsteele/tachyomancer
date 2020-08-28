#version 330 core

in vec2 relativePoint;

uniform vec2 RectSize;
uniform vec4 ShadowColor;
uniform float ShadowRadius;

out vec4 color;

// The next two functions are taken from:
//   http://madebyevan.com/shaders/fast-rounded-rectangle-shadows/
// License: CC0 (http://creativecommons.org/publicdomain/zero/1.0/)

// This approximates the error function, needed for the gaussian integral
vec4 erf(vec4 x) {
  vec4 s = sign(x), a = abs(x);
  x = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
  x *= x;
  return s - s / (x * x);
}

// Return the mask for the shadow of a box from lower to upper
float boxShadow(vec2 lower, vec2 upper, vec2 point, float sigma) {
  vec4 query = vec4(point - lower, point - upper);
  vec4 integral = 0.5 + 0.5 * erf(query * (sqrt(0.5) / sigma));
  return (integral.z - integral.x) * (integral.w - integral.y);
}

void main() {
  float mask = boxShadow(vec2(0, 0), RectSize, relativePoint,
                         0.3 * ShadowRadius);
  color = ShadowColor * mask;
}
