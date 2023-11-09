varying vec2 v_vt;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;

attribute vec2 i_pos;
attribute vec2 i_size;
attribute vec2 i_uv_pos;
attribute vec2 i_uv_size;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_model_matrix;

void main() {
    v_vt = i_uv_pos + a_vt * i_uv_size;
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(i_pos + a_vt * i_size, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform vec4 u_color;
uniform vec4 u_outline_color;
uniform float u_outline_distance;

float antialias(float x) {
    // float w = length(vec2(dFdx(x), dFdy(x)));
    // return 1.0 - smoothstep(-w, w, x);
    return x;
}

float read_sdf(sampler2D text, vec2 uv) {
    return 1.0 - 2.0 * texture2D(text, uv).x;
}

void main() {
    float dist = read_sdf(u_texture, v_vt);

    if (dist > 0.0) {
        discard;
    }
    gl_FragColor = u_color;

    // TODO: fix
    // float inside = antialias(dist);
    // float inside_border = antialias(dist - u_outline_distance);
    // vec4 outside_color = vec4(u_outline_color.xyz, 0.0);
    // gl_FragColor = u_color * inside +
    //     (1.0 - inside) * (
    //         u_outline_color * inside_border +
    //         outside_color * (1.0 - inside_border)
    //     );
}
#endif
