varying vec4 v_color;
varying vec2 v_vt;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec4 a_color;
uniform mat3 u_texture_matrix;
void main() {
    v_color = a_color;
    v_vt = a_vt;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER

uniform vec2 u_framebuffer_size;
uniform vec2 u_pattern_size;
uniform sampler2D u_texture;
uniform sampler2D u_dither1;
uniform sampler2D u_dither2;
uniform sampler2D u_dither3;

void main() {
	vec3 color = (v_color * texture2D(u_texture, v_vt)).rgb;
	vec2 pixel_pos = v_vt * vec2(360.0 * 16.0 / 9.0, 360.0) / u_pattern_size + vec2(0.5) / vec2(360.0 * 16.0 / 9.0, 360.0);
	if (color.r < 0.125) {
		color = vec3(0.0);
	} else if (color.r < 0.125 + 0.25) {
		color = texture2D(u_dither1, pixel_pos).rgb;
	} else if (color.r < 0.125 + 0.5) {
		color = texture2D(u_dither2, pixel_pos).rgb;
	} else if (color.r < 0.125 + 0.75) {
		color = texture2D(u_dither3, pixel_pos).rgb;
	} else {
		color = vec3(1.0);
	}
	// color.r = v_vt.x;
	
    gl_FragColor = vec4(color, 1.0);
    // gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
#endif