varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;
uniform float u_depth;

void main() {
    v_uv = a_vt;
    gl_Position = vec4(a_pos, u_depth, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_mask_texture;
uniform sampler2D u_color_texture;

void main() {
    vec4 mask = texture2D(u_mask_texture, v_uv);
    vec4 color = texture2D(u_color_texture, v_uv);
    color *= mask;
    if (color.a < 0.1) {
        discard;
    }
    gl_FragColor = color;
}
#endif

