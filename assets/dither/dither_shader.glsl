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
uniform vec2 u_framebuffer_size;
uniform vec2 u_pattern_size;

uniform vec4 u_color_dark;
uniform vec4 u_color_light;
uniform vec4 u_color_danger;

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

vec2 get_pixel_pos() {
	return v_vt * u_framebuffer_size / u_pattern_size + vec2(0.5) / u_framebuffer_size;
}

float dither(float amp) {
	vec2 pixel_pos = get_pixel_pos();
	if (amp < 0.125) {
		return 0.0;
	} else if (amp < 0.125 + 0.25) {
		return texture2D(u_dither1, pixel_pos).r;
	} else if (amp < 0.125 + 0.5) {
		return texture2D(u_dither2, pixel_pos).r;
	} else if (amp < 0.125 + 0.75) {
		return texture2D(u_dither3, pixel_pos).r;
	}
	return 1.0;
}

float dither_inverted(float amp) {
	vec2 pixel_pos = v_vt * u_framebuffer_size / u_pattern_size + vec2(0.5) / u_framebuffer_size;
	if (amp < 0.125) {
		return 0.0;
	} else if (amp < 0.125 + 0.25) {
		return 1.0 - texture2D(u_dither3, pixel_pos).r;
	} else if (amp < 0.125 + 0.5) {
		return 1.0 - texture2D(u_dither2, pixel_pos).r;
	} else if (amp < 0.125 + 0.75) {
		return 1.0 - texture2D(u_dither1, pixel_pos).r;
	}
	return 1.0;
}

vec4 dither_final(vec2 amps) {
	float noise_light = 0.1 * (noise(vec3(u_time * 16.0, get_pixel_pos() * 2.0)) * 2.0 - 1.0);
	float noise_danger = 0.1 * (noise(vec3(u_time * 16.0, (get_pixel_pos() + vec2(10.0)) * 2.0)) * 2.0 - 1.0);
	
	float amp_light = amps.g + noise_light;
	float amp_danger = amps.r + noise_danger;
	float total = amp_light + amp_danger;
	if (total > 1.0) {
		amp_light /= total;
		amp_danger /= total;
	}
	
	if (dither_inverted(amp_danger) > 0.0) return u_color_danger;
	if (dither(amp_light) > 0.0) return u_color_light;
	
	return u_color_dark;
}

void main() {
	vec4 in_color = texture2D(u_texture, v_vt);
	gl_FragColor = dither_final(in_color.rg);
}
#endif