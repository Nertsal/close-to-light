pub mod bezier;
pub mod spline;
mod transform;

use self::transform::*;
pub use self::{bezier::*, spline::*};

use super::*;

pub struct Interpolation<T> {
    pub segments: Vec<InterpolationSegment<T>>,
}

impl<T: 'static + Interpolatable> Interpolation<T> {
    pub fn get(&self, mut interval: usize, t: FloatTime) -> Option<T> {
        self.segments
            .iter()
            .find(|segment| {
                let f = segment.num_intervals() > interval;
                if !f {
                    interval -= segment.num_intervals();
                }
                f
            })
            .and_then(|segment| segment.get(interval, t))
    }

    /// Get the full path of the interpolation with the given `resolution` per segment.
    pub fn get_path(&self, resolution: usize) -> impl Iterator<Item = T> + '_ {
        self.segments
            .first()
            .and_then(|segment| segment.get(0, FloatTime::ZERO))
            .into_iter()
            .chain(self.segments.iter().flat_map(move |segment| {
                (0..segment.num_intervals()).flat_map(move |interval| {
                    (1..=resolution).flat_map(move |i| {
                        let t = i as f32 / resolution as f32;
                        segment.get(interval, FloatTime::new(t))
                    })
                })
            }))
    }
}

pub enum InterpolationSegment<T> {
    Linear(Vec<T>), // TODO: smallvec
    Spline(Spline<T>),
    Bezier(Bezier<T>),
}

impl<T: 'static + Interpolatable> InterpolationSegment<T> {
    pub fn linear(points: &[T]) -> Self {
        Self::Linear(points.to_vec())
    }

    pub fn spline(points: &[T], tension: f32) -> Self {
        Self::Spline(Spline::new(points, tension))
    }

    pub fn bezier(points: &[T]) -> Self {
        Self::Bezier(Bezier::new(points))
    }

    /// The number of intervals within the curve segment.
    pub fn num_intervals(&self) -> usize {
        match self {
            Self::Linear(points) => points.len().saturating_sub(1),
            Self::Spline(spline) => spline.num_intervals(),
            Self::Bezier(bezier) => bezier.num_intervals(),
        }
    }

    /// Get an interpolated value on the given interval.
    pub fn get(&self, interval: usize, t: FloatTime) -> Option<T> {
        match self {
            Self::Linear(points) => {
                let a = points.get(interval)?.clone();
                let b = points.get(interval + 1)?.clone();
                Some(a.clone().add(b.sub(a).scale(t.as_f32()))) // a + (b - a) * t
            }
            Self::Spline(i) => i.get(interval, t),
            Self::Bezier(i) => i.get(interval, t),
        }
    }
}

pub trait Interpolatable: Clone {
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn scale(self, factor: f32) -> Self;
    fn length(self) -> f32 {
        self.length_sqr().sqrt()
    }
    fn length_sqr(self) -> f32;
}

impl<T: Float> Interpolatable for Angle<T> {
    fn add(self, other: Self) -> Self {
        self + other
    }
    fn sub(self, other: Self) -> Self {
        self - other
    }
    fn scale(self, factor: f32) -> Self {
        self * T::from_f32(factor)
    }
    fn length(self) -> f32 {
        self.as_radians().as_f32()
    }
    fn length_sqr(self) -> f32 {
        self.as_radians().as_f32().sqr()
    }
}

impl<A: Interpolatable, B: Interpolatable> Interpolatable for (A, B) {
    fn add(self, other: Self) -> Self {
        (self.0.add(other.0), self.1.add(other.1))
    }
    fn sub(self, other: Self) -> Self {
        (self.0.sub(other.0), self.1.sub(other.1))
    }
    fn scale(self, factor: f32) -> Self {
        (self.0.scale(factor), self.1.scale(factor))
    }
    fn length_sqr(self) -> f32 {
        self.0.length_sqr() + self.1.length_sqr()
    }
}

macro_rules! impl_interpolatable_for_float {
    ($T:ty) => {
        impl Interpolatable for $T {
            fn add(self, other: Self) -> Self {
                <$T as Add>::add(self, other)
            }
            fn sub(self, other: Self) -> Self {
                <$T as Sub>::sub(self, other)
            }
            fn scale(self, factor: f32) -> Self {
                <$T>::mul(self, <$T as Float>::from_f32(factor))
            }
            fn length(self) -> f32 {
                <$T as Float>::as_f32(self)
            }
            fn length_sqr(self) -> f32 {
                <$T as Float>::as_f32(self).sqr()
            }
        }
    };
}

macro_rules! impl_interpolatable_for_vec {
    ($T:ident) => {
        impl<T: Interpolatable + Copy> Interpolatable for $T<T> {
            fn add(self, other: Self) -> Self {
                $T::from(std::array::from_fn(|i| self[i].add(other[i])))
            }
            fn sub(self, other: Self) -> Self {
                $T::from(std::array::from_fn(|i| self[i].sub(other[i])))
            }
            fn scale(self, factor: f32) -> Self {
                self.map(|x| x.scale(factor))
            }
            fn length_sqr(self) -> f32 {
                self.iter().map(|x| x.length_sqr()).sum::<f32>()
            }
        }
    };
}

macro_rules! impl_interpolatable_for_mat {
    ($T:ident) => {
        impl<T: Interpolatable + Copy> Interpolatable for $T<T> {
            fn add(self, other: Self) -> Self {
                $T::new(std::array::from_fn(|i| {
                    std::array::from_fn(|j| self[(i, j)].add(other[(i, j)]))
                }))
            }
            fn sub(self, other: Self) -> Self {
                $T::new(std::array::from_fn(|i| {
                    std::array::from_fn(|j| self[(i, j)].sub(other[(i, j)]))
                }))
            }
            fn scale(self, factor: f32) -> Self {
                self.map(|x| x.scale(factor))
            }
            fn length_sqr(self) -> f32 {
                self.as_flat_array()
                    .iter()
                    .map(|x| x.length().sqr())
                    .sum::<f32>()
            }
        }
    };
}

impl_interpolatable_for_float!(f32);
impl_interpolatable_for_float!(f64);
impl_interpolatable_for_float!(R32);
impl_interpolatable_for_float!(R64);

impl_interpolatable_for_vec!(vec2);
impl_interpolatable_for_vec!(vec3);
impl_interpolatable_for_vec!(vec4);

impl_interpolatable_for_mat!(mat3);
impl_interpolatable_for_mat!(mat4);

// #[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
// #[serde(untagged)]
// pub enum InterpolationSerde {
//     Prefab(InterpolationPrefab),
//     Raw(Interpolation),
// }

// #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
// pub enum Interpolation {
//     Linear,
//     Smoothstep,
//     Spline,
//     Bezier,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// #[serde(from = "InterpolationSerde")]
// pub struct Interpolation {}

// impl From<InterpolationSerde> for Interpolation {
//     fn from(value: InterpolationSerde) -> Self {
//         match value {
//             InterpolationSerde::Prefab(prefab) => prefab.into(),
//             InterpolationSerde::Raw(interpolation) => interpolation,
//         }
//     }
// }

// impl From<InterpolationPrefab> for Interpolation {
//     fn from(value: InterpolationPrefab) -> Self {
//         match value {
//             InterpolationPrefab::Linear => todo!(),
//             InterpolationPrefab::Smoothstep => todo!(),
//         }
//     }
// }

// #[test]
// fn parse_interpolation() {
//     fn prefab(s: &str, prefab: InterpolationPrefab) {
//         assert_eq!(
//             serde_json::from_str::<InterpolationPrefab>(s).unwrap(),
//             prefab
//         );
//         assert_eq!(
//             serde_json::from_str::<InterpolationSerde>(s).unwrap(),
//             InterpolationSerde::Prefab(prefab)
//         );
//         assert_eq!(
//             serde_json::from_str::<Interpolation>(s).unwrap(),
//             Interpolation::from(prefab)
//         );
//     }

//     prefab(r#""Linear""#, InterpolationPrefab::Linear);
//     prefab(r#""Smoothstep""#, InterpolationPrefab::Smoothstep);
// }
