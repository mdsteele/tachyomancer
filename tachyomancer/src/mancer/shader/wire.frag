#version 330 core

in float fragment_tex_param;

uniform vec3 WireColor;
uniform sampler1D WireTexture;
uniform vec4 HilightColor;

out vec4 color;

void main() {
  vec4 tex = texture(WireTexture, fragment_tex_param);
  float wire_brightness = tex.r;
  float wire_alpha = tex.g;
  float hilight_brightness = tex.b;
  float hilight_alpha = tex.a;
  color = vec4(WireColor * wire_brightness +
               HilightColor.rgb * hilight_brightness,
               wire_alpha + HilightColor.a * hilight_alpha);
}
