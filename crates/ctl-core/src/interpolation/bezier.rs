use super::*;

/// A single Bezier segment of arbitrary degree, defined by the given number of points.
pub struct Bezier<T> {
    points: Vec<T>, // TODO: smallvec
    uniform_transform: Box<dyn Fn(f32) -> f32>,
}

impl<T: 'static + Interpolatable> Bezier<T> {
    pub fn new(points: &[T]) -> Self {
        // To avoid NaN issues with uniform transform calculation
        if points.len() <= 2 {
            return Self {
                points: points.to_vec(),
                uniform_transform: Box::new(|t| t),
            };
        }

        let sample = |t: f32| sample(points, t);
        let uniform_transform = Box::new(calculate_uniform_transformation(&sample));
        Self {
            points: points.to_vec(),
            uniform_transform,
        }
    }

    pub fn num_intervals(&self) -> usize {
        self.points.len().saturating_sub(1)
    }

    /// Returns a smoothed point.
    /// `t` is expected to be in range `0..=1`.
    pub fn get(&self, interval: usize, t: FloatTime) -> Option<T> {
        let degree = self.points.len().saturating_sub(1);
        if degree == 0 {
            return self.points.first().cloned();
        }

        let t = (t.as_f32() + interval as f32) / degree as f32;
        let t = (self.uniform_transform)(t);
        Some(sample(&self.points, t))
    }
}

fn sample<T: Interpolatable>(points: &[T], t: f32) -> T {
    let n = points
        .len()
        .checked_sub(1)
        .expect("cannot sample an empty array");
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
