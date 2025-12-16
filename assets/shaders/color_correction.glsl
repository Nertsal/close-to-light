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
uniform float u_saturation;

void main() {
    vec4 color = texture2D(u_texture, v_vt);

    // Saturation reduction
    // Greyscale by relative luminance <https://www.davetech.co.uk/shaderdesaturate>
    float greyscale = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    color.rgb = mix(vec3(greyscale), color.rgb, u_saturation);

    gl_FragColor = color;
}
#endif
