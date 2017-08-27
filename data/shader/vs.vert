#version 330

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coords;

out vec3 f_normal;
out vec2 f_tex_coords;

void main() {
  f_normal = normal;
  f_tex_coords = tex_coords;

  gl_Position = projection * view * model * vec4(position,  1.0);
}
