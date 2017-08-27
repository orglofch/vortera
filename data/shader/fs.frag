#version 330

uniform sampler2D diffuse_texture;

in vec3 f_normal;
in vec2 f_tex_coords;

out vec4 out_colour;

void main() {
  vec4 diffuse_colour = texture(diffuse_texture, f_tex_coords);

  out_colour = diffuse_colour;//vec4(abs(f_normal), 1.0);
}
