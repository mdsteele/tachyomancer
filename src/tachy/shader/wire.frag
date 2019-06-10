#version 330 core

in float fragment_tex_param;

uniform vec3 WireColor;
uniform sampler1D WireTexture;
uniform float Hilight;

out vec4 color;

const vec3 HilightColor = vec3(0.592, 0.949, 0.988);

void main() {
  vec4 tex = texture(WireTexture, fragment_tex_param);
  color = vec4(WireColor.rgb * tex.r + HilightColor * (Hilight * tex.b),
               tex.g + Hilight * tex.a);
}
