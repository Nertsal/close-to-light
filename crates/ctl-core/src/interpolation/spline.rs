use super::*;

/// Represents a [cardinal spline](https://en.wikipedia.org/wiki/Cubic_Hermite_spline#Cardinal_spline).
pub struct Spline<T> {
    intervals: Vec<Interval<T>>, // TODO: smallvec
}

/// Represents a single interval of the spline.
struct Interval<T> {
    /// Starting point
    pub point_start: T,
    /// End point
    pub point_end: T,
    /// Starting tangent
    pub tangent_start: T,
    /// End tangent
    pub tangent_end: T,
    /// Transformation to apply to input time for the spline to output uniform speed.
    pub uniform_transformation: Box<dyn Fn(f32) -> f32>,
}

impl<T: Interpolatable> Interval<T> {
    /// Returns a point on the curve interval.
    pub fn get(&self, t: Time) -> T {
        let p0 = self.point_start.clone();
        let p1 = self.point_end.clone();
        let m0 = self.tangent_start.clone();
        let m1 = self.tangent_end.clone();

        let t = (self.uniform_transformation)(t.as_f32());
        sample(t, p0, p1, m0, m1)
    }
}

impl<T: 'static + Interpolatable> Spline<T> {
    pub fn num_intervals(&self) -> usize {
        self.intervals.len()
    }

    /// Create a cardinal spline passing through points.
    /// Tension should be in range `0..=1`.
    /// For example, if tension is `0.5`, then the curve is a Catmull-Rom spline.
    pub fn new(points: &[T], tension: f32) -> Self {
        let n = points.len();
        let intervals = intervals(points, tension);
        assert_eq!(n, intervals.len() + 1);
        Self { intervals }
    }

    /// Returns a point on the spline on the given interval.
    /// Returns `None` if there are no points.
    pub fn get(&self, interval: usize, t: Time) -> Option<T> {
        let interval = self.intervals.get(interval)?;
        Some(interval.get(t))
    }
}

fn intervals<T: 'static + Interpolatable>(points: &[T], tension: f32) -> Vec<Interval<T>> {
    // Calculate tangents
    let len = points.len();
    let mut tangents = Vec::with_capacity(len);
    if len > 1 {
        let p0 = points[0].clone();
        let p1 = points[1].clone();
        let tangent = p1.clone().sub(p0.clone()).scale(1.0 - tension); // (1.0 - t) / (1.0 - 0.0)

        if len == 2 {
            return vec![Interval {
                point_start: p0,
                point_end: p1,
                tangent_start: tangent.clone(),
                tangent_end: tangent,
                uniform_transformation: Box::new(|t| t),
            }];
        }

        tangents.push((0, tangent));
    }
    tangents.extend(
        points
            .iter()
            .zip(points.iter().skip(2))
            .map(|(p0, p2)| p2.clone().sub(p0.clone()).scale(1.0 - tension)) // (1.0 - t) / (1.0 - 0.0)
            .enumerate()
            .map(|(i, m)| (i + 1, m)),
    );
    if len > 2 {
        tangents.push((
            len - 1,
            points[len - 1]
                .clone()
                .sub(points[len - 2].clone())
                .scale(1.0 - tension), // (1.0 - t) / (1.0 - 0.0)
        ));
    }

    // Convert to intervals
    let mut tangents = tangents.into_iter();

    let (_, mut prev) = match tangents.next() {
        Some(first) => first,
        None => return Vec::new(),
    };

    let mut intervals = Vec::with_capacity(len - 1);
    for (index, next) in tangents {
        let p0 = points[index - 1].clone();
        let p1 = points[index].clone();
        let m0 = prev;
        let m1 = next.clone();

        let sample = |t| sample(t, p0.clone(), p1.clone(), m0.clone(), m1.clone());
        let uniform_transformation = Box::new(calculate_uniform_transformation(&sample));

        intervals.push(Interval {
            point_start: p0,
            point_end: p1,
            tangent_start: m0,
            tangent_end: m1,
            uniform_transformation,
        });
        prev = next;
    }

    intervals
}

fn sample<T: Interpolatable>(t: f32, p0: T, p1: T, m0: T, m1: T) -> T {
    let t2 = t * t; // t^2
    let t3 = t2 * t; // t^3
    let one = 1.0;
    let two = 2.0;
    let three = 3.0;
    p0.scale(two * t3 - three * t2 + one)
        .add(m0.scale(t3 - two * t2 + t))
        .add(p1.scale(-two * t3 + three * t2))
        .add(m1.scale(t3 - t2))
}
