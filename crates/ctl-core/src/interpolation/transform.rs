use super::*;

/// Calculate the transformation that should be used with the
/// `sample` function to achieve uniform speed when interpolating.
pub fn calculate_uniform_transformation<T: Interpolatable>(
    sample: &impl Fn(f32) -> T,
) -> impl Fn(f32) -> f32 {
    const RESOLUTION: usize = 100;
    let step = (RESOLUTION as f32).recip();

    // Approximate the integral - curve length
    // Compute the underlying part at some resolution
    let mut total_length = R32::ZERO;
    let mut last_sample = sample(0.0);
    let mut integral: [R32; RESOLUTION] = std::array::from_fn(|i| {
        let t = (i + 1) as f32 * step;
        let s = sample(t);
        // NOTE: no need to multiply gradient by the step size, because we normalize later anyway
        total_length += r32(s.clone().sub(last_sample.clone()).length());
        last_sample = s;
        total_length
    });

    // Normalize so it ranges from 0 to 1
    for d in &mut integral {
        *d /= total_length;
    }

    move |t: f32| {
        let i = match integral.binary_search(&r32(t)) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };

        if i >= RESOLUTION - 1 {
            return 1.0;
        }

        let min = integral[i].as_f32();
        let max = integral[i + 1].as_f32();
        let interp = (t - min) / (max - min);

        ((i + 1) as f32 + interp) * step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    fn check<T: Interpolatable>(
        sample: &impl Fn(f32) -> T,
        transform: &impl Fn(f32) -> f32,
        resolution: usize,
        max_deviation: f32,
    ) {
        let step = (resolution as f32).recip();
        let (min, max) = (0..=resolution)
            .map(|i| {
                let t = i as f32 * step;
                let t = transform(t);
                sample(t)
            })
            .tuple_windows()
            .map(|(a, b)| b.sub(a).length())
            .filter(|x| *x > 0.0)
            .minmax()
            .into_option()
            .expect("not a single element");
        assert!(
            max - min <= max_deviation,
            "deviation too large (expected {}): min step size was {}, max step size was {}",
            max_deviation,
            min,
            max
        );
    }

    #[test]
    fn test_interpolation_uniform_cubic() {
        fn sample(t: f32) -> vec2<f32> {
            vec2(
                t * t - 5.0 * t + 10.0,
                1.7 * t * t * t + 1.4 * t * t - 0.5 * t,
            )
        }
        let transform = calculate_uniform_transformation(&sample);
        check(&sample, &transform, 10, 0.0266);
        check(&sample, &transform, 100, 0.000355);
    }
}
