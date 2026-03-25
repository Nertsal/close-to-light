varying vec2 v_direction;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
attribute mat3 i_model_matrix;

attribute vec2 a_pos;
attribute vec2 i_direction;

void main() {
    v_vt = a_pos;
    v_direction = i_direction;
    vec3 pos = u_projection_matrix * u_view_matrix * i_model_matrix * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    if (length(v_vt) > 1.0) {
        discard;
    }
    gl_FragColor = vec4(v_direction * 0.5 + 0.5, 0.0, 1.0);
}
#endif


