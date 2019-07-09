#version 330 core

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in VS_OUT {
  vec2 textureUV;
} vs_out[];

out GS_OUT {
  vec2 textureUV;
  float brightness;
} gs_out;

const vec3 lightDir = normalize(vec3(0.6, -0.8, 1.0));

void main() {
  vec3 v0 = vec3(gl_in[0].gl_Position);
  vec3 v1 = vec3(gl_in[1].gl_Position);
  vec3 v2 = vec3(gl_in[2].gl_Position);
  vec3 normal = normalize(cross(v1 - v0, v2 - v0));

  float diffuse = abs(dot(normal, lightDir));
  gs_out.brightness = 0.7 * diffuse + 0.3;

  gs_out.textureUV = vs_out[0].textureUV;
  gl_Position = vec4(v0, 1);
  EmitVertex();

  gs_out.textureUV = vs_out[1].textureUV;
  gl_Position = vec4(v1, 1);
  EmitVertex();

  gs_out.textureUV = vs_out[2].textureUV;
  gl_Position = vec4(v2, 1);
  EmitVertex();

  EndPrimitive();
}
