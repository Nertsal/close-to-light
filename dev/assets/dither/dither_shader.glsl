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
uniform vec4 u_bg_color;
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

vec3 rgb2hsv(vec3 c) {
    vec4 k = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, k.wz), vec4(c.gb, k.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float dither(float amp) {
	vec2 pixel_pos = v_vt * u_framebuffer_size / u_pattern_size + vec2(0.5) / u_framebuffer_size;
	vec3 color;
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
	return color.r; // Assume gray-scale
}

vec4 dither(float amp, vec3 light_color) {
	float r = dither(amp * light_color.r);
	float g = dither(amp * light_color.g);
	float b = dither(amp * light_color.b);
	float t = max(max(r, g), b);

	vec4 dark = u_bg_color;
	// vec4 light = vec4(light_color, 1.0);
	vec4 light = vec4(r, g, b, 1.0);
	return dark + (light - dark) * t;
}

void main() {
	vec4 in_color = texture2D(u_texture, v_vt);
	// float v = max(max(in_color.r, in_color.g), in_color.b);
	// in_color = vec4(in_color.rgb + 1.0 - v, in_color.a); // Undo overflow protection

	vec2 pixel_pos = v_vt * u_framebuffer_size / u_pattern_size + vec2(0.5) / u_framebuffer_size;

	// vec3 hsv = rgb2hsv(in_rgb);
	// Pattern based on alpha
	// float amp = hsv.z;

	// Noise
	float amp = 0.1 * (noise(vec3(u_time * 16.0, pixel_pos * 2.0)) * 2.0 - 1.0);

	// vec2 r = dither(in_color.r + amp).ra;
	// vec2 g = dither(in_color.g + amp).ga;
	// vec2 b = dither(in_color.b + amp).ba;
	// float a = max(max(r.y, g.y), b.y);

	// Lerp from dark to light
	// vec3 light = vec3(r.x, g.x, b.x);
	// vec3 dark = u_bg_color.rgb;
	// float t = in_color.a; //max(max(light.x, light.y), light.z);
	// vec3 color = dark + (light - dark) * t;

	vec3 light_color = rgb2hsv(in_color.rgb);
	if (light_color.z > 0.0) {
		light_color.z = 1.0; // Value always 1 unless it is 0 (black)
	}
	light_color = hsv2rgb(light_color);
	vec4 color = dither(in_color.a + amp, light_color);
	// vec4 color = dither(in_color.a + amp, in_color.rgb);

	// Change saturation
	// hsv.y = rgb2hsv(color.rgb).y;

    // gl_FragColor = vec4(hsv2rgb(hsv), color.a);
    // gl_FragColor = vec4(in_color.rgb, 1.0) * color;
	gl_FragColor = color;
}
#endif