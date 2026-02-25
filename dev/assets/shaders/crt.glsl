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
uniform float u_curvature;
uniform float u_vignette_multiplier;
uniform float u_scanlines_multiplier;
uniform float u_time;

void main() {
    vec2 centered_uv = v_vt * 2.0 - 1.0;
    vec2 uv_offset = centered_uv.yx / u_curvature;
    vec2 warped_uv =
        centered_uv
        + centered_uv * uv_offset * uv_offset
        + step(centered_uv.x + centered_uv.y * 0.15, sin(u_time * 0.3) * 1.2) * 0.002;
    vec3 cutoff = vec3(step(abs(warped_uv.x), 1.0) * step(abs(warped_uv.y), 1.0));
    float scanlines = sin(2.0 * warped_uv.y * 180.0 + mod(u_time, 3.14159) * 2.0);
    vec3 vignette = vec3(length(pow(abs(centered_uv), vec2(4.0)) / 3.0));

    vec3 texel = texture2D(u_texture, (warped_uv + 1.0) / 2.0, 0.2).rgb;
    float dark = min(texel.r, min(texel.g, texel.b));
    float light = max(texel.r, max(texel.g, texel.b));

    // NOTE: offset light level to make scanlines more noticable on pure black and white
    float light_offset = 0.0;
    float threshold = 0.2 * u_scanlines_multiplier;
    if (dark < threshold) {
        light_offset = threshold - dark;
    } else if (dark > 1.0 - threshold) {
        light_offset = 1.0 - threshold - dark;
    }

    vec3 screen_color =
        texel * cutoff
        + vec3(light_offset)
        + vec3(0.05) * scanlines * u_scanlines_multiplier;
    screen_color -= vignette * u_vignette_multiplier;
    gl_FragColor = vec4(screen_color, 1.0);
}
#endif
