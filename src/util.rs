use ctl_font::Font;
use ctl_logic::Vec2RealConversions;
pub use ctl_util::*;
use geng::prelude::*;

pub fn lerp<T: ctl_logic::Float>(min: T, max: T, t: T) -> T {
    min + (max - min) * t
}

pub fn wrap_text(font: &Font, text: &str, target_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut line = String::new();
        for word in source_line.split_whitespace() {
            if line.is_empty() {
                line += word;
                continue;
            }
            if font.measure(&(line.clone() + " " + word), 1.0).width() > target_width {
                lines.push(line);
                line = word.to_string();
            } else {
                line += " ";
                line += word;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

/// Tiles the `area` with the subtexture uv's.
pub fn tile_area_subtexture<T: Float>(
    sub_texture: Aabb2<f32>,
    tile_size: vec2<T>,
    offset: vec2<T>,
    area: Aabb2<T>,
) -> Vec<draw2d::TexturedVertex> {
    let tile_size = tile_size.map(T::as_f32);
    let offset = offset.map(T::as_f32);
    let area = area.map(T::as_f32);

    let [a, b, c, d] = geng_utils::geometry::unit_quad();
    let [va, vb, vc, vd] = sub_texture.corners();
    let set_uv = |v, uv| draw2d::TexturedVertex { a_vt: uv, ..v };
    let unit = [set_uv(a, va), set_uv(b, vb), set_uv(c, vc), set_uv(d, vd)];

    let tiles = (area.size() / tile_size).map(|x| x.ceil() as usize);
    (0..tiles.x)
        .flat_map(|x| {
            (0..tiles.y).flat_map(move |y| {
                let mut vs = unit.map(|v| draw2d::TexturedVertex {
                    a_pos: area.bottom_left()
                        + offset
                        + (vec2(x, y).as_f32() + (v.a_pos + vec2(1.0, 1.0)) / 2.0) * tile_size,
                    ..v
                });

                // cut out overflow
                if vs[2].a_pos.x > area.max.x {
                    let overflow = (vs[2].a_pos.x - area.max.x) / tile_size.x * sub_texture.width();
                    vs[1].a_pos.x = area.max.x;
                    vs[2].a_pos.x = area.max.x;
                    vs[1].a_vt.x -= overflow;
                    vs[2].a_vt.x -= overflow;
                }
                if vs[2].a_pos.y > area.max.y {
                    let overflow =
                        (vs[2].a_pos.y - area.max.y) / tile_size.y * sub_texture.height();
                    vs[2].a_pos.y = area.max.y;
                    vs[3].a_pos.y = area.max.y;
                    vs[2].a_vt.y -= overflow;
                    vs[3].a_vt.y -= overflow;
                }

                let indices = [0, 1, 2, 0, 2, 3];
                indices.map(|i| vs[i])
            })
        })
        .collect()
}
