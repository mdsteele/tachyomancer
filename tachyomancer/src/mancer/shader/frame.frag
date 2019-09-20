#version 330 core

in vec2 textureUV;

uniform uint Grayscale;
uniform sampler2DMS Texture;

out vec4 color;

void main() {
  ivec2 size = textureSize(Texture);
  ivec2 coords = ivec2(floor(textureUV.x * size.x),
                       floor(textureUV.y * size.y));
  vec4 c0 = 0.25 * texelFetch(Texture, coords, 0);
  vec4 c1 = 0.25 * texelFetch(Texture, coords, 1);
  vec4 c2 = 0.25 * texelFetch(Texture, coords, 2);
  vec4 c3 = 0.25 * texelFetch(Texture, coords, 3);
  vec4 avg = c0 + c1 + c2 + c3;
  if (Grayscale != 0u) {
    float gray = (avg.r + avg.g + avg.b) / 3.0;
    color = vec4(gray, gray, gray, avg.a);
  } else {
    color = avg;
  }
}
