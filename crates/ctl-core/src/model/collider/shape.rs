use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Shape {
    Circle { radius: Coord },
    Line { width: Coord },
    Rectangle { width: Coord, height: Coord },
}

impl Shape {
    pub fn circle(radius: Coord) -> Self {
        Self::Circle { radius }
    }

    pub fn line(width: Coord) -> Self {
        Self::Line { width }
    }

    pub fn rectangle(size: vec2<Coord>) -> Self {
        Self::Rectangle {
            width: size.x,
            height: size.y,
        }
    }

    pub fn to_parry(self) -> Box<dyn parry2d::shape::Shape> {
        match self {
            Shape::Circle { radius } => Box::new(parry2d::shape::Ball::new(radius.as_f32())),
            Shape::Line { width } => Shape::Rectangle {
                width: r32(1e3), // TODO: unhack
                height: width,
            }
            .to_parry(),
            Shape::Rectangle { width, height } => {
                if width == R32::ZERO || height == R32::ZERO {
                    return Box::new(parry2d::shape::Ball::new(0.0));
                }
                let aabb = Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0);
                let points = aabb.corners().map(|p| {
                    let vec2(x, y) = p;
                    parry2d::math::Point::new(x, y)
                });
                match parry2d::shape::ConvexPolygon::from_convex_hull(&points) {
                    Some(shape) => Box::new(shape),
                    None => Box::new(parry2d::shape::Ball::new(0.0)),
                }
            }
        }
    }

    pub fn scaled(self, scale: Coord) -> Self {
        match self {
            Shape::Circle { radius } => Shape::Circle {
                radius: radius * scale,
            },
            Shape::Line { width } => Shape::Line {
                width: width * scale,
            },
            Shape::Rectangle { width, height } => Shape::Rectangle {
                width: width * scale,
                height: height * scale,
            },
        }
    }
}
