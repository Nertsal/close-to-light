#![allow(dead_code)]

use geng::prelude::*;

pub use geng_utils::layout::*;

pub type Area = Aabb2<f32>;

pub trait AreaOps {
    fn get(&self) -> Area;

    fn set(&mut self, s: Area);

    fn square_longside(&self) -> Area {
        let area = self.get();
        let d = area.width() - area.height();
        if d > 0.0 {
            Area {
                min: vec2(area.min.x, area.min.y - d / 2.0),
                max: vec2(area.max.x, area.max.y + d / 2.0),
            }
        } else {
            let d = -d;
            Area {
                min: vec2(area.min.x - d / 2.0, area.min.y),
                max: vec2(area.max.x + d / 2.0, area.max.y),
            }
        }
    }

    fn square_shortside(&self) -> Area {
        let area = self.get();
        let d = area.width() - area.height();
        if d > 0.0 {
            Area {
                min: vec2(area.min.x + d / 2.0, area.min.y),
                max: vec2(area.max.x - d / 2.0, area.max.y),
            }
        } else {
            let d = -d;
            Area {
                min: vec2(area.min.x, area.min.y + d / 2.0),
                max: vec2(area.max.x, area.max.y - d / 2.0),
            }
        }
    }

    fn zero_size(&self, align: vec2<f32>) -> Area {
        Area::point(self.align_pos(align))
    }

    fn cut_left(&mut self, width: f32) -> Area {
        let left = self.get().extend_right(width - self.get().width());
        self.set(self.get().extend_left(-width));
        left
    }

    fn split_left(&mut self, ratio: f32) -> Area {
        self.cut_left(self.get().width() * ratio)
    }

    fn cut_right(&mut self, width: f32) -> Area {
        let right = self.get().extend_left(width - self.get().width());
        self.set(self.get().extend_right(-width));
        right
    }

    fn split_right(&mut self, ratio: f32) -> Area {
        self.cut_right(self.get().width() * ratio)
    }

    fn cut_top(&mut self, height: f32) -> Area {
        let top = self.get().extend_down(height - self.get().height());
        self.set(self.get().extend_up(-height));
        top
    }

    fn split_top(&mut self, ratio: f32) -> Area {
        self.cut_top(self.get().height() * ratio)
    }

    fn cut_bottom(&mut self, height: f32) -> Area {
        let bottom = self.get().extend_up(height - self.get().height());
        self.set(self.get().extend_down(-height));
        bottom
    }

    fn split_bottom(&mut self, ratio: f32) -> Area {
        self.cut_bottom(self.get().height() * ratio)
    }

    fn split_rows(&self, rows: usize) -> Vec<Area> {
        let row_height = self.get().height() / rows as f32;
        (0..rows)
            .map(|i| {
                Aabb2::point(self.get().top_left() - vec2(0.0, row_height * (i + 1) as f32))
                    .extend_positive(vec2(self.get().width(), row_height))
            })
            .collect()
    }

    fn split_columns(&self, columns: usize) -> Vec<Area> {
        let column_width = self.get().width() / columns as f32;
        (0..columns)
            .map(|i| {
                Aabb2::point(self.get().bottom_left() + vec2(column_width * i as f32, 0.0))
                    .extend_positive(vec2(column_width, self.get().height()))
            })
            .collect()
    }

    fn stack(&self, offset: vec2<f32>, cells: usize) -> Vec<Area> {
        (0..cells)
            .map(|i| self.get().translate(offset * i as f32))
            .collect()
    }

    fn stack_aligned(&self, offset: vec2<f32>, cells: usize, align: vec2<f32>) -> Vec<Area> {
        let mut cells = self.stack(offset, cells);
        let mut total = self.get();
        if let Some(last) = cells.last() {
            total.min.x = total.min.x.min(last.min.x);
            total.min.y = total.min.y.min(last.min.y);
            total.max.x = total.max.x.max(last.max.x);
            total.max.y = total.max.y.max(last.max.y);
        }
        for pos in &mut cells {
            *pos = pos.translate(self.get().size() * align - total.size() * align);
        }
        cells
    }

    fn with_width(&self, width: f32, align: f32) -> Area {
        align_aabb(
            vec2(width, self.get().height()),
            self.get(),
            vec2(align, 0.5),
        )
    }

    fn with_height(&self, height: f32, align: f32) -> Area {
        align_aabb(
            vec2(self.get().width(), height),
            self.get(),
            vec2(0.5, align),
        )
    }

    /// Get a point inside the aabb.
    /// (0.0, 0.0) corresponds to min.
    /// (1.0, 1.0) corresponds to max.
    fn align_pos(&self, align: vec2<f32>) -> vec2<f32> {
        self.get().min + self.get().size() * align
    }

    /// Align an aabb of the given size inside this one.
    fn align_aabb(&self, size: vec2<f32>, align: vec2<f32>) -> Area {
        let pos_aabb = self.get().extend_symmetric(-size * 0.5);
        let pos = aabb_pos(pos_aabb, align);
        Aabb2::point(pos).extend_symmetric(size * 0.5)
    }

    /// Fit an aabb of the given size into this one.
    fn fit_aabb(&self, size: vec2<f32>, align: vec2<f32>) -> Area {
        let ratio = self.get().size() / size;
        let ratio = if ratio.x < ratio.y { ratio.x } else { ratio.y };
        let fit_size = size * ratio;
        self.align_aabb(fit_size, align)
    }

    /// Fit an aabb of the given size by width into this one.
    fn fit_aabb_width(&self, size: vec2<f32>, align: f32) -> Area {
        let ratio = self.get().width() / size.x;
        let fit_size = size * ratio;
        self.align_aabb(fit_size, vec2(0.0, align))
    }

    /// Fit an aabb of the given size by height into this one.
    fn fit_aabb_height(&self, size: vec2<f32>, align: f32) -> Area {
        let ratio = self.get().height() / size.y;
        let fit_size = size * ratio;
        self.align_aabb(fit_size, vec2(align, 0.0))
    }
}

impl AreaOps for Area {
    fn get(&self) -> Area {
        *self
    }

    fn set(&mut self, s: Area) {
        *self = s;
    }
}

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Area(pub Aabb2<f32>);

// impl Deref for Area {
//     type Target = Aabb2<f32>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl Area {
//     pub fn cut_left(&mut self, width: f32) -> Area {
//         let left = self.0.extend_right(width - self.0.width());
//         *self = Area(self.0.extend_left(-width));
//         Area(left)
//     }

//     pub fn split_left(&mut self, ratio: f32) -> Area {
//         self.cut_left(self.0.width() * ratio)
//     }

//     pub fn cut_right(&mut self, width: f32) -> Area {
//         let right = self.0.extend_left(-width);
//         *self = Area(self.0.extend_right(width - self.0.width()));
//         Area(right)
//     }

//     pub fn split_right(&mut self, ratio: f32) -> Area {
//         self.cut_right(self.0.width() * ratio)
//     }

//     pub fn cut_top(&mut self, width: f32) -> Area {
//         let top = self.0.extend_down(width - self.0.height());
//         *self = Area(self.0.extend_up(-width));
//         Area(top)
//     }

//     pub fn split_top(&mut self, ratio: f32) -> Area {
//         self.cut_top(self.0.height() * ratio)
//     }

//     pub fn cut_bottom(&mut self, height: f32) -> Area {
//         let bottom = self.0.extend_up(-height);
//         *self = Area(self.0.extend_down(height - self.0.height()));
//         Area(bottom)
//     }

//     pub fn split_bottom(&mut self, ratio: f32) -> Area {
//         self.cut_bottom(self.0.height() * ratio)
//     }

//     pub fn split_rows(self, rows: usize) -> Vec<Area> {
//         let row_height = self.0.height() / rows as f32;
//         (0..rows)
//             .map(|i| {
//                 Area(
//                     Aabb2::point(self.0.bottom_left() + vec2(0.0, row_height * i as f32))
//                         .extend_positive(vec2(self.0.width(), row_height)),
//                 )
//             })
//             .collect()
//     }

//     pub fn split_columns(self, columns: usize) -> Vec<Area> {
//         let column_width = self.0.width() / columns as f32;
//         (0..columns)
//             .map(|i| {
//                 Area(
//                     Aabb2::point(self.0.bottom_left() + vec2(column_width * i as f32, 0.0))
//                         .extend_positive(vec2(column_width, self.0.height())),
//                 )
//             })
//             .collect()
//     }

//     pub fn stack(self, offset: vec2<f32>, cells: usize) -> Vec<Area> {
//         (0..cells)
//             .map(|i| Area(self.0.translate(offset * i as f32)))
//             .collect()
//     }

//     pub fn with_width(self, width: f32, align: f32) -> Area {
//         Area(align_aabb(
//             vec2(width, self.0.height()),
//             self.0,
//             vec2(align, 0.5),
//         ))
//     }

//     pub fn with_height(self, height: f32, align: f32) -> Area {
//         Area(align_aabb(
//             vec2(self.0.width(), height),
//             self.0,
//             vec2(0.5, align),
//         ))
//     }

//     /// Get a point inside the aabb.
//     /// (0.0, 0.0) corresponds to min.
//     /// (1.0, 1.0) corresponds to max.
//     pub fn align_pos(self, align: vec2<f32>) -> vec2<f32> {
//         self.0.min + self.0.size() * align
//     }

//     /// Align an aabb of the given size inside this one.
//     pub fn align_aabb(self, size: vec2<f32>, align: vec2<f32>) -> Area {
//         let pos_aabb = self.0.extend_symmetric(-size * 0.5);
//         let pos = aabb_pos(pos_aabb, align);
//         Area(Aabb2::point(pos).extend_symmetric(size * 0.5))
//     }

//     /// Fit an aabb of the given size into this one.
//     pub fn fit_aabb(self, size: vec2<f32>, align: vec2<f32>) -> Area {
//         let ratio = self.0.size() / size;
//         let ratio = if ratio.x < ratio.y { ratio.x } else { ratio.y };
//         let fit_size = size * ratio;
//         self.align_aabb(fit_size, align)
//     }

//     /// Fit an aabb of the given size by width into this one.
//     pub fn fit_aabb_width(self, size: vec2<f32>, align: f32) -> Area {
//         let ratio = self.0.width() / size.x;
//         let fit_size = size * ratio;
//         self.align_aabb(fit_size, vec2(0.0, align))
//     }

//     /// Fit an aabb of the given size by height into this one.
//     pub fn fit_aabb_height(self, size: vec2<f32>, align: f32) -> Area {
//         let ratio = self.0.height() / size.y;
//         let fit_size = size * ratio;
//         self.align_aabb(fit_size, vec2(align, 0.0))
//     }
// }
