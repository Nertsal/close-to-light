varying vec2 v_vt;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;

void main() {
    v_vt = a_vt;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform vec4 u_color_dark;
uniform vec4 u_color_light;

void main() {
    vec4 in_color = texture2D(u_texture, v_vt);
    if (in_color == vec4(vec3(0.0), 1.0)) {
        gl_FragColor = u_color_dark;
    } else if (in_color == vec4(vec3(1.0), 1.0)) {
        gl_FragColor = u_color_light;
    } else {
        gl_FragColor = in_color;
    }
}
#endif
