use super::*;

/// A Bezier curve of arbitrary degree.
pub struct Bezier<const N: usize, T> {
    segments: Vec<BezierSegment<N, T>>,
}

impl<const N: usize, T: 'static + Interpolatable> Bezier<N, T> {
    pub fn new(points: &[T]) -> Self {
        Self {
            segments: points
                .windows(N)
                .enumerate()
                .filter(|(i, _)| *i % (N - 1) == 0)
                .map(|(_, window)| {
                    assert_eq!(window.len(), N);
                    let points = std::array::from_fn(|i| window[i].clone());
                    BezierSegment::new(points)
                })
                .collect(),
        }
    }

    pub fn get(&self, interval: usize, t: Time) -> Option<T> {
        let subinterval = interval % (N - 1);
        let interval = interval / (N - 1);
        let interval = self.segments.get(interval)?;
        let t = (t.as_f32() + subinterval as f32) / (N - 1) as f32;
        Some(interval.get(t))
    }
}

/// A single Bezier segment of arbitrary degree.
/// BezierSegment<3> is a Bezier defined by 3 points, so a curve of the 2nd degree, or a quadratic Bezier.
pub struct BezierSegment<const N: usize, T> {
    points: [T; N],
    uniform_transform: Box<dyn Fn(f32) -> f32>,
}

impl<const N: usize, T: 'static + Interpolatable> BezierSegment<N, T> {
    pub fn new(points: [T; N]) -> Self {
        let sample = {
            let points = points.clone();
            move |t: f32| sample(&points, t)
        };
        let uniform_transform = Box::new(calculate_uniform_transformation(&sample));
        Self {
            points,
            uniform_transform,
        }
    }

    /// Returns a smoothed point.
    /// `t` is expected to be in range `0..=1`.
    pub fn get(&self, t: f32) -> T {
        let t = (self.uniform_transform)(t);
        sample(&self.points, t)
    }
}

fn sample<const N: usize, T: Interpolatable>(points: &[T; N], t: f32) -> T {
    let n = N - 1;
    (0..=n)
        .map(|i| {
            let p = points[i].clone();
            let c = binomial(n, i) as f32;
            let s = (1.0 - t).powi(n as i32 - i as i32) * t.powi(i as i32);
            p.scale(c * s)
        })
        .reduce(Interpolatable::add)
        .expect("there have to be control points")
}

// copied from crate num_integer::binomial
fn binomial(mut n: usize, k: usize) -> usize {
    // See http://blog.plover.com/math/choose.html for the idea.
    if k > n {
        return 0;
    }
    if k > n - k {
        return binomial(n, n - k);
    }
    let mut r = 1;
    let mut d = 1;
    loop {
        if d > k {
            break;
        }
        r = multiply_and_divide(r, n, d);
        n -= 1;
        d += 1;
    }
    r
}

/// Calculate r * a / b, avoiding overflows and fractions.
///
/// Assumes that b divides r * a evenly.
fn multiply_and_divide(r: usize, a: usize, b: usize) -> usize {
    // See http://blog.plover.com/math/choose-2.html for the idea.
    let g = gcd(r, b);
    r / g * (a / (b / g))
}

/// Calculates the Greatest Common Divisor (GCD) of the number and `other`
#[inline]
fn gcd(mut m: usize, mut n: usize) -> usize {
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
