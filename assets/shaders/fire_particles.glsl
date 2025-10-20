varying vec4 v_color;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
attribute mat3 i_model_matrix;

attribute vec2 a_pos;
attribute vec4 i_color;

void main() {
    v_vt = a_pos;
    v_color = i_color;
    vec3 pos = u_projection_matrix * u_view_matrix * i_model_matrix * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;

void main() {
    if (length(v_vt) > 1.0) {
        discard;
    }
    gl_FragColor = u_color * v_color;
}
#endif


