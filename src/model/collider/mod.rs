mod shape;

pub use self::shape::*;

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Collision {
    pub point: vec2<Coord>,
    /// Normal vector pointing away from the body.
    pub normal: vec2<Coord>,
    pub penetration: Coord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    pub position: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub shape: Shape,
}

impl Collider {
    pub fn new(position: vec2<Coord>, shape: Shape) -> Self {
        Self {
            position,
            rotation: Angle::ZERO,
            shape,
        }
    }

    pub fn transformed(&self, transform: Transform) -> Self {
        Self {
            position: self.position + transform.translation,
            rotation: self.rotation + transform.rotation,
            shape: self.shape.scaled(transform.scale),
        }
    }

    pub fn transform_mat(&self) -> mat3<Coord> {
        mat3::translate(self.position) * mat3::rotate(self.rotation)
    }

    pub fn compute_aabb(&self) -> Aabb2<Coord> {
        let (iso, shape) = self.to_parry();
        let parry2d::bounding_volume::Aabb { mins, maxs } = shape.compute_aabb(&iso);
        Aabb2 {
            min: vec2(mins.x, mins.y).as_r32(),
            max: vec2(maxs.x, maxs.y).as_r32(),
        }
    }

    fn get_iso(&self) -> parry2d::math::Isometry<f32> {
        let vec2(x, y) = self.position.as_f32();
        let angle = self.rotation.as_radians().as_f32();
        parry2d::math::Isometry::new(parry2d::na::Vector2::new(x, y), angle)
    }

    fn to_parry(&self) -> (parry2d::math::Isometry<f32>, Box<dyn parry2d::shape::Shape>) {
        (self.get_iso(), self.shape.to_parry())
    }

    /// Check whether the collider contains the point.
    pub fn contains(&self, point: vec2<Coord>) -> bool {
        // TODO: better
        self.check(&Collider {
            position: point,
            rotation: Angle::ZERO,
            shape: Shape::Circle {
                radius: Coord::ZERO,
            },
        })
    }

    /// Check whether two colliders are intersecting.
    pub fn check(&self, other: &Self) -> bool {
        let delta = (other.position - self.position).as_f32();

        let self_angle = self.rotation.as_radians().as_f32();
        let self_iso = parry2d::math::Isometry::rotation(self_angle);
        let self_shape = self.shape.to_parry();

        let other_angle = other.rotation.as_radians().as_f32();
        let other_iso =
            parry2d::math::Isometry::new(parry2d::na::Vector2::new(delta.x, delta.y), other_angle);
        let other_shape = other.shape.to_parry();

        parry2d::query::intersection_test(&self_iso, &*self_shape, &other_iso, &*other_shape)
            .unwrap()
    }

    /// Return the collision info if the two colliders are intersecting.
    pub fn collide(&self, other: &Self) -> Option<Collision> {
        let delta = (other.position - self.position).as_f32();

        let self_angle = self.rotation.as_radians().as_f32();
        let self_iso = parry2d::math::Isometry::rotation(self_angle);
        let self_shape = self.shape.to_parry();

        let other_angle = other.rotation.as_radians().as_f32();
        let other_iso =
            parry2d::math::Isometry::new(parry2d::na::Vector2::new(delta.x, delta.y), other_angle);
        let other_shape = other.shape.to_parry();

        let prediction = 0.0;
        parry2d::query::contact(
            &self_iso,
            &*self_shape,
            &other_iso,
            &*other_shape,
            prediction,
        )
        .unwrap()
        .map(|contact| {
            let normal = contact.normal1.into_inner();
            let point = contact.point1;
            Collision {
                point: vec2(point.x, point.y).map(Coord::new),
                normal: vec2(normal.x, normal.y).map(Coord::new),
                penetration: Coord::new(-contact.dist),
            }
        })
    }
}
