#vs

vec2[4] CO = vec2[](
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1., -1.),
  vec2(-1.,  1.)
);

void main() {
  gl_Position = vec4(CO[gl_VertexID], 0., 1.);
}

#fs

uniform sampler2D source;

out vec4 frag;

void main() {
  frag = texelFetch(source, ivec2(gl_FragCoord.xy), 0);
}
