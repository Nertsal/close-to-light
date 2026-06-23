varying vec4 v_color;
varying vec2 v_vt;

uniform float u_sdf_pad;

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
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(a_pos / (1.0 - u_sdf_pad), 1.0);
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
    float h_max_pre = (1.0 - u_hollow_cut) / 2.0;
    float hollow_max = u_sdf_pad + (1.0 - u_sdf_pad) * h_max_pre;
    color.a = 1.0 - abs(color.a - hollow_max) / h_max_pre;

    vec4 m_color = u_color * v_color;
    color.rgb *= m_color.rgb;

    color = vec4(color.rgb * color.a, m_color.a); // Premultiply alpha
    gl_FragColor = color;
}
#endif
