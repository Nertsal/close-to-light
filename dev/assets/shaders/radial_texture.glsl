varying vec2 v_pos;
varying vec2 v_vt;
varying vec4 v_color;

#ifdef VERTEX_SHADER
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_model_matrix;

attribute float a_z;
attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec4 a_color;

void main() {
    v_pos = a_pos;
    v_vt = a_vt;
    v_color = a_color;
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, a_z, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform sampler2D u_texture;
uniform float u_angle; // Expected to be in range -pi..pi
uniform float u_angle_range; // Expected to be in range -2pi..2pi

void main() {
    vec4 in_color = texture2D(u_texture, v_vt);
    vec4 color = in_color * u_color * v_color;
    if (color.a < 0.1) {
        discard;
    }

    float from = u_angle;
    float range = u_angle_range;
    if (range < 0.0) {
        from = from + range;
        range = -range;
        if (from < -PI) {
            from += 2.0 * PI;
        }
    }

    float angle = atan(v_pos.y, v_pos.x);
    if ((angle >= from && angle <= from + range)
        || (range > PI - from && angle <= from + range - 2.0 * PI)) {
        gl_FragColor = color;
    } else {
        discard;
    }
}
#endif
