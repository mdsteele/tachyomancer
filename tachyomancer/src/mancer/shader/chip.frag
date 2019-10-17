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
  float labelAlpha = texture(IconTexture, gs_out.textureUV).r;
  vec4 baseColor = vec4(PlasticColor.rgb * gs_out.brightness, PlasticColor.a);
  vec4 labelColor = vec4(IconColor * mix(gs_out.brightness, 1.0, 0.3), 1.0);
  color = mix(baseColor, labelColor, labelAlpha);
}
