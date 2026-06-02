mod lerp;
mod sod;
mod task;

pub use self::{lerp::*, sod::*, task::*};

use geng::prelude::*;
use geng_utils::bounded::Bounded;

pub fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}

/// Returns the given color with the multiplied alpha.
pub fn with_alpha(mut color: Rgba<f32>, alpha: f32) -> Rgba<f32> {
    color.a *= alpha;
    color
}

pub fn world_to_screen(
    camera: &impl geng::AbstractCamera2d,
    framebuffer_size: vec2<f32>,
    pos: vec2<f32>,
) -> vec2<f32> {
    let pos = (camera.projection_matrix(framebuffer_size) * camera.view_matrix()) * pos.extend(1.0);
    let pos = pos.xy() / pos.z;
    vec2(
        (pos.x + 1.0) / 2.0 * framebuffer_size.x,
        (pos.y + 1.0) / 2.0 * framebuffer_size.y,
    )
}

pub fn argsort_by_key<T, K: Ord>(data: &[T], mut f: impl FnMut(&T) -> K) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| f(&data[i]));
    indices
}

/// Calculates the Greatest Common Divisor (GCD) of the number and `other`
pub fn gcd(mut m: usize, mut n: usize) -> usize {
    // Use Stein's algorithm
    if m == 0 || n == 0 {
        return m | n;
    }

    // find common factors of 2
    let shift = (m | n).trailing_zeros();

    // divide n and m by 2 until odd
    m >>= m.trailing_zeros();
    n >>= n.trailing_zeros();

    while m != n {
        if m > n {
            m -= n;
            m >>= m.trailing_zeros();
        } else {
            n -= m;
            n >>= n.trailing_zeros();
        }
    }
    m << shift
}

/// Calculates the Lowest Common Multiple (LCM) of the number and `other`.
pub fn lcm(m: usize, n: usize) -> usize {
    gcd_lcm(m, n).1
}

/// Calculates the Greatest Common Divisor (GCD) and
/// Lowest Common Multiple (LCM) of the number and `other`.
pub fn gcd_lcm(m: usize, n: usize) -> (usize, usize) {
    if m == 0 && n == 0 {
        return (0, 0);
    }
    let gcd = gcd(m, n);
    let lcm = m * (n / gcd);
    (gcd, lcm)
}

/// Change speed of the sound while preserving pitch.
pub async fn change_sound_speed(
    sound: &geng::Sound,
    speed: f32,
    geng: &Geng,
) -> anyhow::Result<geng::Sound> {
    let speed = speed.clamp(0.2, 5.0);
    let sample_rate = sound.sample_rate();
    let channels_n = sound.number_of_channels() as u32;
    assert!(channels_n <= 2);
    let channels: Vec<_> = (0..channels_n).map(|i| sound.get_channel_data(i)).collect();

    let state: Box<[f32; pitch_shift::TOTAL_F32]> =
        vec![0.0; pitch_shift::TOTAL_F32].try_into().unwrap();
    let mut shifter = pitch_shift::Shifter::new(state);
    const IN_SAMPLES: usize = 128;
    let out_samples: usize = (IN_SAMPLES as f32 * speed) as usize;

    let mut samples = Vec::with_capacity(channels_n as usize);
    for channel in channels {
        let (chunks, _remainder) = channel.as_chunks::<IN_SAMPLES>();
        let mut out_channel = Vec::with_capacity(chunks.len() * out_samples);
        for chunk in chunks {
            let out_chunk = shifter.shift(chunk, 0.0, out_samples, sample_rate);
            out_channel.extend(out_chunk.iter().copied());
        }
        samples.push(out_channel);
    }

    let sound = geng.audio().sound_from_buffer(samples, sample_rate).await?;
    Ok(sound)
}
