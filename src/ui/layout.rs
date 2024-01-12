#![allow(dead_code)]

use geng::prelude::*;

pub use geng_utils::layout::*;

type Area = Aabb2<f32>;

pub fn split_left_right(aabb: Area, left_ratio: f32) -> (Area, Area) {
    cut_left_right(aabb, aabb.width() * left_ratio)
}

pub fn cut_left_right(aabb: Area, left_width: f32) -> (Area, Area) {
    (
        aabb.extend_right(left_width - aabb.width()),
        aabb.extend_left(-left_width),
    )
}

pub fn split_top_down(aabb: Area, top_ratio: f32) -> (Area, Area) {
    cut_top_down(aabb, aabb.height() * top_ratio)
}

pub fn cut_top_down(aabb: Area, top_height: f32) -> (Area, Area) {
    (
        aabb.extend_down(top_height - aabb.height()),
        aabb.extend_up(-top_height),
    )
}

pub fn split_rows(aabb: Area, rows: usize) -> Vec<Area> {
    let row_height = aabb.height() / rows as f32;
    (0..rows)
        .map(|i| {
            Area::point(aabb.bottom_left() + vec2(0.0, row_height * i as f32))
                .extend_positive(vec2(aabb.width(), row_height))
        })
        .collect()
}

pub fn split_columns(aabb: Area, columns: usize) -> Vec<Area> {
    let column_width = aabb.width() / columns as f32;
    (0..columns)
        .map(|i| {
            Area::point(aabb.bottom_left() + vec2(column_width * i as f32, 0.0))
                .extend_positive(vec2(column_width, aabb.height()))
        })
        .collect()
}

pub fn stack(cell: Area, offset: vec2<f32>, cells: usize) -> Vec<Area> {
    (0..cells)
        .map(|i| cell.translate(offset * i as f32))
        .collect()
}

pub fn with_width(aabb: Area, width: f32, align: f32) -> Area {
    align_aabb(vec2(width, aabb.height()), aabb, vec2(align, 0.5))
}

pub fn with_height(aabb: Area, height: f32, align: f32) -> Area {
    align_aabb(vec2(aabb.width(), height), aabb, vec2(0.5, align))
}
