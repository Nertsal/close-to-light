varying vec4 v_color;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec4 a_color;
uniform mat3 u_texture_matrix;
void main() {
    v_color = a_color;
    v_vt = a_vt;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
void main() {
    gl_FragColor = v_color * texture2D(u_texture, v_vt);
    // gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
#endif