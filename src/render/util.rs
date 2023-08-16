use super::*;

pub struct UtilRender {
    geng: Geng,
    assets: Rc<Assets>,
}

impl UtilRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw_collider(
        &self,
        collider: &Collider,
        color: Rgba<f32>,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::TexturedQuad::colored(
                        Aabb2::ZERO.extend_symmetric(vec2(radius.as_f32(), radius.as_f32())),
                        &self.assets.radial_gradient,
                        color,
                    )
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Line { width } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::TexturedQuad::colored(
                        Aabb2::ZERO.extend_symmetric(vec2(camera.fov * 4.0, width.as_f32()) / 2.0),
                        &self.assets.linear_gradient,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Rectangle { width, height } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Quad::new(
                        Aabb2::ZERO.extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0),
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
    }

    pub fn draw_outline(
        &self,
        collider: &Collider,
        outline_width: f32,
        color: Rgba<f32>,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Ellipse::circle_with_cut(
                        collider.position.as_f32(),
                        radius.as_f32() - outline_width,
                        radius.as_f32(),
                        color,
                    ),
                );
            }
            Shape::Line { width } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(
                            vec2(-camera.fov * 2.0, (width.as_f32() - outline_width) / 2.0),
                            vec2(camera.fov * 2.0, (width.as_f32() - outline_width) / 2.0),
                        ),
                        outline_width,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(
                            vec2(-camera.fov * 2.0, -(width.as_f32() - outline_width) / 2.0),
                            vec2(camera.fov * 2.0, -(width.as_f32() - outline_width) / 2.0),
                        ),
                        outline_width,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Rectangle { width, height } => {
                let [a, b, c, d] = Aabb2::ZERO
                    .extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0)
                    .extend_uniform(-outline_width / 2.0)
                    .corners();
                let m = (a + b) / 2.0;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Chain::new(
                        Chain::new(vec![m, b, c, d, a, m]),
                        outline_width,
                        color,
                        1,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
    }
}
