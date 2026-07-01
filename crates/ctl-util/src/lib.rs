mod lerp;
mod sod;
mod task;

pub use self::{lerp::*, sod::*, task::*};

use ctl_core::types::{FloatTime, Time, seconds_to_time, time_to_seconds};
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

pub fn display_time(time: Time, include_ms: bool) -> String {
    let mut ms = time;
    let mut secs = ms / 1000;
    ms -= secs * 1000;
    let mins = secs / 60;
    secs -= mins * 60;
    if include_ms {
        format!("{:02}:{:02}.{:03}", mins, secs, ms)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

pub struct TimeInterpolation {
    state: SecondOrderState<FloatTime>,
    pub value: Time,
    pub target: Time,
}

impl Default for TimeInterpolation {
    fn default() -> Self {
        Self::new(3.0)
    }
}

impl TimeInterpolation {
    pub fn new(lerp_speed: f32) -> Self {
        let time = Time::ZERO;
        Self {
            state: SecondOrderState::new(lerp_speed, 1.0, 0.0, time_to_seconds(time)),
            value: time,
            target: time,
        }
    }

    pub fn update(&mut self, delta_time: FloatTime) {
        self.state.update(delta_time.as_f32());
        if (self.state.current - self.state.target).abs().as_f32() < 0.002 {
            // Skip the final step for better precision on dependent visuals
            self.state.current = self.state.target;
        }
        self.value = seconds_to_time(self.state.current);
    }

    pub fn scroll_time(&mut self, change: Change<Time>) {
        change.apply(&mut self.target);
        self.state.target = time_to_seconds(self.target);
    }

    pub fn snap_to(&mut self, time: Time) {
        self.value = time;
        self.target = time;
        let time = time_to_seconds(self.value);
        self.state.current = time;
        self.state.target = time;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Change<T> {
    Add(T),
    Set(T),
}

impl<T: Sub<Output = T>> Change<T> {
    pub fn into_delta(self, reference_value: T) -> T {
        match self {
            Change::Add(delta) => delta,
            Change::Set(target_value) => target_value.sub(reference_value),
        }
    }
}

impl<T: Add<Output = T> + Copy> Change<T> {
    pub fn apply(&self, value: &mut T) {
        *value = match *self {
            Change::Add(delta) => value.add(delta),
            Change::Set(value) => value,
        };
    }
}

impl<T: PartialEq> Change<T> {
    pub fn is_noop(&self, zero_delta: &T) -> bool {
        match self {
            Change::Add(delta) => delta == zero_delta,
            Change::Set(_) => false,
        }
    }
}
