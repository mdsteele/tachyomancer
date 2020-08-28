#version 330 core

in GS_OUT {
  vec2 textureUV;
  float brightness;
} gs_out;

uniform vec4 PlasticColor;
uniform vec3 IconColor;
uniform sampler2D IconTexture;

out vec4 color;

void main() {
  float labelBrightness = mix(gs_out.brightness, 1.0, 0.3);
  float labelAlpha = texture(IconTexture, gs_out.textureUV).r;
  vec4 labelColor = vec4(IconColor * labelBrightness * labelAlpha, labelAlpha);
  vec4 baseColor = vec4(PlasticColor.rgb * gs_out.brightness, PlasticColor.a);
  color = labelColor + baseColor * (1.0 - labelColor.a);
}
