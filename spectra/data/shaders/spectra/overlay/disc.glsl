#vs
layout (location = 0) in vec3 ctr;
layout (location = 1) in vec4 color;
layout (location = 2) in float radius;

out float v_radius;
out vec4 v_color;

void main() {
  gl_Position = vec4(ctr, 1.);
  v_radius = radius;
  v_color = color;
}

#gs

layout (points) in;
layout (triangle_strip, max_vertices = 6) out;

uniform float ratio;

in float v_radius[];
in vec4 v_color[];

out vec2 g_nco; // normalized coordinates
out vec4 g_color;

void main() {
  vec2 p_0 = gl_in[0].gl_Position.xy + vec2(-v_radius[0], -v_radius[0] * ratio);
  vec2 p_1 = gl_in[0].gl_Position.xy + vec2(v_radius[0], -v_radius[0] * ratio);
  vec2 p_2 = gl_in[0].gl_Position.xy + vec2(v_radius[0], v_radius[0] * ratio);
  vec2 p_3 = gl_in[0].gl_Position.xy + vec2(-v_radius[0], v_radius[0] * ratio);

  gl_Position = vec4(p_1, 0., 1.);
  g_nco = vec2(1., -1.);
  g_color = v_color[0];
  EmitVertex();

  gl_Position = vec4(p_2, 0., 1.);
  g_nco = vec2(1., 1.);
  g_color = v_color[0];
  EmitVertex();

  gl_Position = vec4(p_0, 0., 1.);
  g_nco = vec2(-1., -1.);
  g_color = v_color[0];
  EmitVertex();

  gl_Position = vec4(p_3, 0., 1.);
  g_nco = vec2(-1., 1.);
  g_color = v_color[0];
  EmitVertex();

  EndPrimitive();
}

#fs
in vec2 g_nco;
in vec4 g_color;

out vec4 frag;

void main() {
  if (length(g_nco) > 1.) { // out, discard
    discard;
  }

  frag = g_color;
}
