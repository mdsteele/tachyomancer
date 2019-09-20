#version 330 core

in VS_OUT {
  vec3 normalCamSpace;
  vec3 materialColor;
  vec2 textureUV;
} vs_out;

uniform float AmbientLight;
uniform float DiffuseLight;
uniform vec3 LightDirCamSpace;
uniform sampler2D Texture;

out vec4 color;

void main() {
  vec4 baseColor = texture(Texture, vs_out.textureUV) *
    vec4(vs_out.materialColor, 1);
  float brightness = AmbientLight +
    DiffuseLight * clamp(dot(vs_out.normalCamSpace, LightDirCamSpace), 0, 1);
  color = vec4(baseColor.rgb * brightness, baseColor.a);
}
