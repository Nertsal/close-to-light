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
uniform float u_affect_color;
uniform float u_t;

// Green and red channels are inverted sdf representations of white and red lights.
// 1.0 means center of the light, 0.75 and lower means no light, but still encoded relative distance.
void main() {
    vec4 mask = texture2D(u_mask_texture, v_uv);
    float mask_value = max(mask.g, mask.r);

    vec4 color = texture2D(u_color_texture, v_uv);
    color = vec4(
        mix(color.rgb, color.rgb * mask_value, u_affect_color * u_t),
        mix(color.a, color.a * mask_value, u_t)
    );
    gl_FragColor = color;
}
#endif

