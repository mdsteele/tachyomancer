#version 330 core

in VS_OUT {
  vec3 normalCamSpace;
  vec3 materialColor;
} vs_out;

uniform float AmbientLight;
uniform float DiffuseLight;
uniform vec3 LightDirCamSpace;

out vec4 color;

void main() {
  float brightness = AmbientLight +
    DiffuseLight * clamp(dot(vs_out.normalCamSpace, LightDirCamSpace), 0, 1);
  color = vec4(vs_out.materialColor * brightness, 1);
}
