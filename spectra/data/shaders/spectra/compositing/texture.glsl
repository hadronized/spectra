#vs

out vec2 v_co;

vec2[4] CO = vec2[](
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1., -1.),
  vec2(-1.,  1.)
);

void main() {
  vec2 co = CO[gl_VertexID];
  gl_Position = vec4(co, 0., 1.);
  v_co = (1. + co) * .5;
}

#fs

uniform sampler2D source;
uniform vec2 scale;

in vec2 v_co;
out vec4 frag;

void main() {
  frag = texture(source, v_co * scale);
}

