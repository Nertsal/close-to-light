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
uniform float u_rgb_offset;
uniform float u_noise_offset;
uniform float u_time;

// white noise
float noise(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    const float DIV = 160.0;
    vec2 noise_sample = ceil(v_vt * DIV) / DIV + u_time;
    vec2 vt = v_vt + vec2(noise(noise_sample) - 0.5, 0.0) * (1.0 / DIV) * u_noise_offset;

    vec4 color = texture2D(u_texture, vt);
    vec2 rb_offset = vec2(1.0, 0.2) * u_rgb_offset;
    vec4 color_left = texture2D(u_texture, vt - rb_offset);
    vec4 color_right = texture2D(u_texture, vt + rb_offset);
    color.r = color_left.r;
    color.b = color_right.b;

    gl_FragColor = color;
}
#endif

