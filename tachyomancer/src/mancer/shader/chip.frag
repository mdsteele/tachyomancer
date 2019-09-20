#version 330 core

in GS_OUT {
  vec2 textureUV;
  float brightness;
} gs_out;

uniform vec3 IconColor;
uniform sampler2D IconTexture;

out vec4 color;

void main() {
  float alpha = texture(IconTexture, gs_out.textureUV).r;
  vec3 plasticColor = vec3(0.4, 0.4, 0.4) * gs_out.brightness;
  vec3 labelColor = IconColor * mix(gs_out.brightness, 1.0, 0.3);
  vec3 combinedColor = mix(plasticColor, labelColor, alpha);
  color = vec4(combinedColor, 1);
}
