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
uniform float u_offset;
uniform float u_time;

void main() {
    vec4 color = texture2D(u_texture, v_vt);
    vec2 offset = vec2(1.0, 0.2) * u_offset;
    vec4 color_left = texture2D(u_texture, v_vt - offset);
    vec4 color_right = texture2D(u_texture, v_vt + offset);
    color.r = color_left.r;
    color.b = color_right.b;
    gl_FragColor = color;
}
#endif

