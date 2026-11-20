/** `w` of the heights/widths/angles texel of travels and wipes: this plus their `EOptionType` index, above the role ids. */
export const KIND_OPTION = 32;

// libvgcode `Segments_Vertex_Shader_ES` (`ShadersES.hpp:13-148`, POINTY_CAPS and
// FIX_TWISTING): instance `n` is the enabled segment `id_a = aSegment[n]` to
// `id_b = id_a + 1` (`segment_index_tex`, `ShadersES.hpp:94-97`), whose ends are
// fetched from the per-vertex textures. The `w` of the position texel holds the
// vertex layer and that of the heights/widths/angles texel the role or option kind,
// for the layer and visibility filter. The template vertex id arrives as
// `position.x`; `cameraPosition` and `viewMatrix` are world space because the
// toolpath mesh has an identity model matrix (`ViewerImpl.cpp:1300-1302`).
export const vertexShader = /* glsl */ `
#define POINTY_CAPS
#define FIX_TWISTING
const vec3  light_top_dir = vec3(-0.4574957, 0.4574957, 0.7624929);
const float light_top_diffuse = 0.6 * 0.8;
const float light_top_specular = 0.6 * 0.125;
const float light_top_shininess = 20.0;
const vec3  light_front_dir = vec3(0.6985074, 0.1397015, 0.6985074);
const float light_front_diffuse = 0.6 * 0.3;
const float ambient = 0.3;
const float emission = 0.15;
const vec3 UP = vec3(0, 0, 1);
uniform sampler2D position_tex;
uniform sampler2D height_width_angle_tex;
uniform sampler2D color_tex;
in uint aSegment;
uniform vec2 uLayers;
uniform int uHiddenRoles;
uniform int uOptions;
out vec3 color;

float lighting(vec3 eye_position, vec3 eye_normal) {
  float top_diffuse = light_top_diffuse * max(dot(eye_normal, light_top_dir), 0.0);
  float front_diffuse = light_front_diffuse * max(dot(eye_normal, light_front_dir), 0.0);
  float top_specular = light_top_specular * pow(max(dot(-normalize(eye_position), reflect(-light_top_dir, eye_normal)), 0.0), light_top_shininess);
  return ambient + top_diffuse + front_diffuse + top_specular + emission;
}

ivec2 tex_coord(sampler2D sampler, int id) {
  ivec2 tex_size = textureSize(sampler, 0);
  return (tex_size.y == 1) ? ivec2(id, 0) : ivec2(id % tex_size.x, id / tex_size.x);
}

bool hidden(float layer, int kind) {
  if (layer < uLayers.x || layer > uLayers.y) return true;
  if (kind >= ${KIND_OPTION}) return ((uOptions >> (kind - ${KIND_OPTION})) & 1) == 0;
  return ((uHiddenRoles >> kind) & 1) == 1;
}

void main() {
  int id_a = int(aSegment);
  int id_b = id_a + 1;
  vec4 position_layer_b = texelFetch(position_tex, tex_coord(position_tex, id_b), 0);
  vec4 height_width_angle_kind_b = texelFetch(height_width_angle_tex, tex_coord(height_width_angle_tex, id_b), 0);
  if (hidden(position_layer_b.w, int(height_width_angle_kind_b.w))) {
    gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
    return;
  }
  int vertex_id = int(position.x);
  vec3 pos_a = texelFetch(position_tex, tex_coord(position_tex, id_a), 0).xyz;
  vec3 pos_b = position_layer_b.xyz;
  vec3 line = pos_b - pos_a;
  // directions of the line box in world space
  float line_len = length(line);
  vec3 line_dir;
  if (line_len < 1e-4)
    line_dir = vec3(1.0, 0.0, 0.0);
  else
    line_dir = line / line_len;
  vec3 line_right_dir;
  if (abs(dot(line_dir, UP)) > 0.9) {
    // For vertical lines, the width and height should be same, there is no concept of up and down.
    // For simplicity, the code will expand width in the x axis, and height in the y axis
    line_right_dir = normalize(cross(vec3(1, 0, 0), line_dir));
  }
  else
    line_right_dir = normalize(cross(line_dir, UP));
  vec3 line_up_dir = normalize(cross(line_right_dir, line_dir));
  const vec2 horizontal_vertical_view_signs_array[16] = vec2[](
    //horizontal view (from right)
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    vec2(0.0, -1.0),
    vec2(0.0, -1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    // vertical view (from top)
    vec2(0.0, 1.0),
    vec2(-1.0, 0.0),
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(-1.0, 0.0),
    vec2(0.0, 0.0)
    );
  vec3 hwa_a = texelFetch(height_width_angle_tex, tex_coord(height_width_angle_tex, id_a), 0).xyz;
  vec3 hwa_b = height_width_angle_kind_b.xyz;
  vec3 endpoint_pos = vertex_id < 4 ? pos_a : pos_b;
  vec3 height_width_angle = vertex_id < 4 ? hwa_a : hwa_b;
#ifdef FIX_TWISTING
  bool closer_a = dot(cameraPosition - pos_a, cameraPosition - pos_a) < dot(cameraPosition - pos_b, cameraPosition - pos_b);
  vec3 closer_pos = closer_a ? pos_a : pos_b;
  vec3 camera_view_dir = normalize(closer_pos - cameraPosition);
  vec3 closer_height_width_angle = closer_a ? hwa_a : hwa_b;
  vec3 diagonal_dir_border = normalize(closer_height_width_angle.x * line_up_dir + closer_height_width_angle.y * line_right_dir);
#else
  vec3 camera_view_dir = normalize(endpoint_pos - cameraPosition);
  vec3 diagonal_dir_border = normalize(height_width_angle.x * line_up_dir + height_width_angle.y * line_right_dir);
#endif
  bool is_vertical_view = abs(dot(camera_view_dir, line_up_dir)) / abs(dot(diagonal_dir_border, line_up_dir)) >
    abs(dot(camera_view_dir, line_right_dir)) / abs(dot(diagonal_dir_border, line_right_dir));
  vec2 signs = horizontal_vertical_view_signs_array[vertex_id + 8 * int(is_vertical_view)];
#ifndef POINTY_CAPS
  if (vertex_id == 2 || vertex_id == 7) signs = -horizontal_vertical_view_signs_array[(vertex_id - 2) + 8 * int(is_vertical_view)];
#endif
  float view_right_sign = sign(dot(-camera_view_dir, line_right_dir));
  float view_top_sign = sign(dot(-camera_view_dir, line_up_dir));
  float half_height = 0.5 * height_width_angle.x;
  float half_width = 0.5 * height_width_angle.y;
  vec3 horizontal_dir = half_width * line_right_dir;
  vec3 vertical_dir = half_height * line_up_dir;
  float horizontal_sign = signs.x * view_right_sign;
  float vertical_sign = signs.y * view_top_sign;
  vec3 pos = endpoint_pos + horizontal_sign * horizontal_dir + vertical_sign * vertical_dir;
  if (vertex_id == 2 || vertex_id == 7) {
    float line_dir_sign = (vertex_id == 2) ? -1.0 : 1.0;
    if (height_width_angle.z == 0.0) {
#ifdef POINTY_CAPS
      // There I add a cap to lines that do not have a following line
      // (or they have one, but perfectly aligned, so the cap is hidden inside the next line).
      pos += line_dir_sign * line_dir * half_width;
#endif
    }
    else {
      pos += line_dir_sign * line_dir * half_width * sin(abs(height_width_angle.z) * 0.5);
      pos += sign(height_width_angle.z) * horizontal_dir * cos(abs(height_width_angle.z) * 0.5);
    }
  }
  vec3 eye_position = (viewMatrix * vec4(pos, 1.0)).xyz;
  vec3 eye_normal = (viewMatrix * vec4(normalize(pos - endpoint_pos), 0.0)).xyz;
  int color_id = vertex_id < 4 ? id_a : id_b;
  color = texelFetch(color_tex, tex_coord(color_tex, color_id), 0).rgb * lighting(eye_position, eye_normal);
  gl_Position = projectionMatrix * vec4(eye_position, 1.0);
}
`;

// libvgcode `Segments_Fragment_Shader_ES` (`ShadersES.hpp:150-157`).
export const fragmentShader = /* glsl */ `
in vec3 color;
out vec4 fragment_color;
void main() {
  fragment_color = vec4(color, 1.0);
}
`;
