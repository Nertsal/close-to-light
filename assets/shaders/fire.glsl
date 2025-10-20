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
uniform float u_time;
uniform sampler2D u_texture;

vec3 permute(vec3 x) { return mod(((x*34.0)+1.0)*x, 289.0); }

// Simplex 2D noise
float noise(vec2 v){
    const vec4 C = vec4(0.211324865405187, 0.366025403784439,
             -0.577350269189626, 0.024390243902439);
    vec2 i  = floor(v + dot(v, C.yy) );
    vec2 x0 = v - i + dot(i, C.xx);
    vec2 i1;
    i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod(i, 289.0);
    vec3 p = permute( permute( i.y + vec3(0.0, i1.y, 1.0 ))
        + i.x + vec3(0.0, i1.x, 1.0 ));
    vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy),
        dot(x12.zw,x12.zw)), 0.0);
    m = m*m ;
    m = m*m ;
    vec3 x = 2.0 * fract(p * C.www) - 1.0;
    vec3 h = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );
    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}

float fire_noise(vec2 p) {
    float x = noise(p) * 0.4 + 0.4;
    if (x < 0.5) {
        return 4.0 * x * x * x;
    } else {
        float x = -2.0 * x + 2.0;
        return 1.0 - x * x * x / 2.0;
    }
}

/// Based on <https://github.com/xxidbr9/balatro-effect-recreate/blob/master/fire-number.gdshader>
float fire_effect(float density) {
    vec2 fire_speed = vec2(0.0, 1.0);
    float fire_aperture = 0.22;
    vec2 uv = vec2(v_vt.x, 1.0 - max(v_vt.y, 0.0));

    // Scale UVs to make the noise more visible
    vec2 base_uv = uv * 5.0;
    
    // Create two layers of noise with different speeds
    vec2 shifted_uv1 = base_uv + u_time * fire_speed; // TODO: becomes slight jagged after a loooong time
    vec2 shifted_uv2 = base_uv + u_time * fire_speed * 1.5;
    
    // Sample noise texture twice
    float fire_noise1 = fire_noise(shifted_uv1) * density;
    float fire_noise2 = fire_noise(shifted_uv2) * density;
    
    // Combine the noise samples
    float combined_noise = (fire_noise1 + fire_noise2) * 0.5;
    
    // Calculate fire shape
    float noise_value = uv.y * (((uv.y + fire_aperture) * combined_noise - fire_aperture) * 25.0);
    
    // Add horizontal movement
    noise_value += sin(uv.y * 10.0 + u_time * 2.0) * 0.1;

    return noise_value;
}

void main() {
    vec4 m_color = texture2D(u_texture, v_vt);

    float fire = fire_effect(m_color.a) * 0.5;
    vec4 fire_color = vec4(m_color.rgb * clamp(fire * 0.08, 0.3, 1.0), clamp(fire, 0.0, 1.0));

    // m_color *= u_color * v_color;
    vec4 color = m_color + fire_color;
    color = vec4(color.rgb * color.a, m_color.a); // Premultiply alpha

    // color = vec4(fire_noise(v_vt * 1.5 + u_time * vec2(0.0, 1.0)), 0.0, 0.0, 1.0);

    gl_FragColor = color;
}
#endif

