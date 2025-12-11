varying vec4 v_color;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_model_matrix;

attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec4 a_color;

void main() {
    v_vt = a_vt;
    v_color = a_color;
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform sampler2D u_texture;
uniform float u_hollow_cut;

void main() {
    vec4 color = texture2D(u_texture, v_vt);

    // Hollow cut
    // NOTE: comparison not with -1.0 due to floating point imprecision in interpolation.
    if (u_hollow_cut > -0.999) {
        color.a = min(
            min(color.a / (1.0 - u_hollow_cut), 1.0),
            max((1.0 - color.a - u_hollow_cut) / (1.0 - u_hollow_cut), 0.0)
        );
    }

    vec4 m_color = u_color * v_color;
    color.rgb *= m_color.rgb;

    color = vec4(color.rgb * color.a, m_color.a); // Premultiply alpha
    gl_FragColor = color;
}
#endif
