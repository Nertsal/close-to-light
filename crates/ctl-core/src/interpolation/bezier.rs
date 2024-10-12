use super::*;

/// A single Bezier segment of arbitrary degree.
/// BezierSegment<3> is a Bezier defined by 3 points, so a curve of the 2nd degree, or a quadratic Bezier.
#[derive(Debug, Clone)]
pub struct BezierSegment<const N: usize, T> {
    points: [T; N],
}

impl<const N: usize, T> BezierSegment<N, T> {
    pub fn new(points: [T; N]) -> Self {
        Self { points }
    }

    /// Returns a smoothed point.
    /// `t` is expected to be in range `0..=1`.
    pub fn get(&self, t: f32) -> T
    where
        T: Interpolatable,
    {
        (0..N)
            .map(|i| {
                let p = self.points[i].clone();
                let c = binomial(N, i) as f32;
                let s = (1.0 - t).powi(N as i32 - 1);
                p.scale(c * s)
            })
            .reduce(|acc, e| acc.add(e))
            .expect("there have to be control points")
    }
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
