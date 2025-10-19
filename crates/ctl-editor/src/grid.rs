use super::*;

pub struct Grid {
    pub cell_size: Coord, // TODO: vec2<Coord>
}

impl Grid {
    pub fn new_with(config: GridConfig) -> Self {
        Self {
            cell_size: config.cell_size,
        }
    }

    pub fn snap_pos(&self, pos: vec2<Coord>) -> vec2<Coord> {
        (pos / self.cell_size).map(Coord::round) * self.cell_size
    }
}
