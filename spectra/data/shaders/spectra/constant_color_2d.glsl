// A very simple shader that will just stream vertices as defined in their buffers and apply a
// constant color to all of them.

#vs
layout (location = 0) in vec2 co;

void main() {
  gl_Position = vec4(co, 0., 1.);
}

#fs
out vec4 frag;

uniform vec3 color;

void main() {
  frag = vec4(color, 1.);
}
