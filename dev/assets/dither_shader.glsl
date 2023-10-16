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

uniform float u_time;
uniform float u_bg_noise;
uniform vec2 u_framebuffer_size;
uniform vec2 u_pattern_size;
uniform sampler2D u_texture;
uniform sampler2D u_dither1;
uniform sampler2D u_dither2;
uniform sampler2D u_dither3;

//	<https://www.shadertoy.com/view/4dS3Wd>
//	By Morgan McGuire @morgan3d, http://graphicscodex.com
//
float hash(float n) { return fract(sin(n) * 1e4); }
float hash(vec2 p) { return fract(1e4 * sin(17.0 * p.x + p.y * 0.1) * (0.1 + abs(sin(p.y * 13.0 + p.x)))); }

float noise(float x) {
	float i = floor(x);
	float f = fract(x);
	float u = f * f * (3.0 - 2.0 * f);
	return mix(hash(i), hash(i + 1.0), u);
}

float noise(vec2 x) {
	vec2 i = floor(x);
	vec2 f = fract(x);

	// Four corners in 2D of a tile
	float a = hash(i);
	float b = hash(i + vec2(1.0, 0.0));
	float c = hash(i + vec2(0.0, 1.0));
	float d = hash(i + vec2(1.0, 1.0));

	// Simple 2D lerp using smoothstep envelope between the values.
	// return vec3(mix(mix(a, b, smoothstep(0.0, 1.0, f.x)),
	//			mix(c, d, smoothstep(0.0, 1.0, f.x)),
	//			smoothstep(0.0, 1.0, f.y)));

	// Same code, with the clamps in smoothstep and common subexpressions
	// optimized away.
	vec2 u = f * f * (3.0 - 2.0 * f);
	return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

// This one has non-ideal tiling properties that I'm still tuning
float noise(vec3 x) {
	const vec3 step = vec3(110, 241, 171);

	vec3 i = floor(x);
	vec3 f = fract(x);
 
	// For performance, compute the base input to a 1D hash from the integer part of the argument and the 
	// incremental change to the 1D based on the 3D -> 1D wrapping
    float n = dot(i, step);

	vec3 u = f * f * (3.0 - 2.0 * f);
	return mix(mix(mix( hash(n + dot(step, vec3(0, 0, 0))), hash(n + dot(step, vec3(1, 0, 0))), u.x),
                   mix( hash(n + dot(step, vec3(0, 1, 0))), hash(n + dot(step, vec3(1, 1, 0))), u.x), u.y),
               mix(mix( hash(n + dot(step, vec3(0, 0, 1))), hash(n + dot(step, vec3(1, 0, 1))), u.x),
                   mix( hash(n + dot(step, vec3(0, 1, 1))), hash(n + dot(step, vec3(1, 1, 1))), u.x), u.y), u.z);
}

void main() {
	vec3 color = (v_color * texture2D(u_texture, v_vt)).rgb;
	vec2 pixel_pos = v_vt * vec2(360.0 * 16.0 / 9.0, 360.0) / u_pattern_size + vec2(0.5) / vec2(360.0 * 16.0 / 9.0, 360.0);
	
	float amp = color.r;
	amp += 0.1 * (noise(vec3(u_time * 16.0, pixel_pos * 2.0)) * 2.0 - 1.0);
	// float mul = max(0.0, (length(v_vt - 0.5) - 0.3) / 0.3 * 2.0);
	float range = u_bg_noise * 2.0 - 1.0;
	float mul = length(v_vt - vec2(0.5)) * 2.0;
	amp += pow(0.02 * mul, mix(1.5 - u_bg_noise, 1.0, 0.4));
#if 1
	if (amp < 0.125) {
		color = vec3(0.0);
	} else if (amp < 0.125 + 0.25) {
		color = texture2D(u_dither1, pixel_pos).rgb;
	} else if (amp < 0.125 + 0.5) {
		color = texture2D(u_dither2, pixel_pos).rgb;
	} else if (amp < 0.125 + 0.75) {
		color = texture2D(u_dither3, pixel_pos).rgb;
	} else {
		color = vec3(1.0);
	}
#else
	color = vec3(amp);
#endif
	
    gl_FragColor = vec4(color, 1.0);
}
#endif