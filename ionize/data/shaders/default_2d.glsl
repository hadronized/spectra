#vs
layout (location = 0) in vec2 co;

void main() {
  gl_Position = vec4(co, 0., 1.);
}

#fs
out vec4 frag;

uniform vec4 color;

void main() {
  frag = vec4(color);
}
