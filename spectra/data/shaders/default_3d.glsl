#vs

layout (location = 0) in vec3 co;

uniform mat4 inst;

void main() {
  gl_Position = inst * vec4(co, 1.);
}

#gs

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

uniform mat4 proj;
uniform mat4 view;

out vec3 g_baryctr;

void main() {
  gl_Position = proj * view * gl_in[0].gl_Position;
  g_baryctr = vec3(1., 0., 0.);
  EmitVertex();
  gl_Position = proj * view * gl_in[1].gl_Position;
  g_baryctr = vec3(0., 1., 0.);
  EmitVertex();
  gl_Position = proj * view * gl_in[2].gl_Position;
  g_baryctr = vec3(0., 0., 1.);
  EmitVertex();

  EndPrimitive();
}

#fs

in vec3 g_baryctr;

out vec4 frag;

void main() {
  float pi = 3.14159265;
  float d = min(min(g_baryctr.x, g_baryctr.y), g_baryctr.z);
  float m = max(max(g_baryctr.x, g_baryctr.y), g_baryctr.z);
  vec4 color = vec4(.6, 0., .5, 1.);
  float k = clamp(cos(8. * 2. * pi * m), 0., 1.);

  if (d < 0.01) {
    color = color * (1. - k) + vec4(.4, 1., .5, 1.) * k;
  }

  frag = color;
}
