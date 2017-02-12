#vs

out vec2 v_uv;

uniform vec3 pos;
uniform vec2 size;
uniform float scale;

const vec2[4] UV = vec2[](
  vec2(1., 1.),
  vec2(1., 0.),
  vec2(0., 1.),
  vec2(0., 0.)
);

void main() {
  //vec2 size = vec2(.3, .1) * 2.;
  vec2[4] verts = vec2[](
    vec2(size.x, -size.y),
    vec2(size.x, size.y),
    vec2(-size.x, -size.y),
    vec2(-size.x, size.y)
  );

  // convert pos into screenspace coordinates
  vec2 screen_pos = pos.xy * 2. - 1.;
  vec4 p = vec4((verts[gl_VertexID] + size) * scale + screen_pos, pos.z, 1.);
  gl_Position = p;

  v_uv = UV[gl_VertexID];
}

#fs

in vec2 v_uv;
out vec4 frag;

uniform sampler2D text_texture;
uniform float scale;
uniform vec4 color;

void main() {
  uint lod = uint(2. - scale * scale * 2.);
  float texel = textureLod(text_texture, v_uv, 0).r;
  vec4 color = color * texel;
  frag = vec4(color);
}
