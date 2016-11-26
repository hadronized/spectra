#vs

layout (location = 0) in vec3 co;
layout (location = 1) in vec3 no;

out vec3 v_p;
out vec3 v_no;

uniform mat4 proj;
uniform mat4 view;
uniform mat4 inst;

void main() {
  mat4 view_inst = view * inst;
  mat4 normal_mat = transpose(inverse(view_inst));

  gl_Position = proj * view_inst * vec4(co, 1.);

  v_p = (inst * vec4(co, 1.)).xyz;
  v_no = (normal_mat * vec4(no, 1.)).xyz;
}

#fs

in vec3 v_p;
in vec3 v_no;

out vec4 frag;

uniform vec3 color;

void main() {
  vec3 light_dir = vec3(0., 1., 0.);
  float kd = max(0., dot(v_no, light_dir));

  frag = vec4(color * kd, 1.);
}
