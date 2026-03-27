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
    float alpha = texture2D(u_texture, v_vt).a;

    // Hollow cut
    float d = alpha * 2.0 / (1.0 - u_hollow_cut); // alpha / hollow_max
    alpha = min(
        d, // Outer
        2.0 - d // Inner
    );

    vec4 m_color = u_color * v_color;
    vec4 color = vec4(m_color.rgb, alpha);

    color = vec4(color.rgb * color.a, m_color.a); // Premultiply alpha

    gl_FragColor = color;
}
#endif
