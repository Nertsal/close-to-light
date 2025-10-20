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
uniform float u_fire;

void main() {
    float alpha = texture2D(u_texture, v_vt).a;
    // NOTE: light gradient texture is assumed to be fully filled,
    // but the coordinates passed into the shader cover the light with extra margin
    // to account for the fire effect.
    // alpha = max(
    //     (alpha - 0.666) / 0.333, // Light itself
    //     alpha * u_fire * 0.5 // Fire
    // );
    alpha = alpha * u_fire;

    vec4 color = u_color * v_color;
    color = vec4(color.rgb * alpha, color.a); // Premultiply alpha

    gl_FragColor = color;
}
#endif

