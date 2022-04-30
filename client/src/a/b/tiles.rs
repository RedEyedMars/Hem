use crate::a::b::Wall;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Tile {
    Empty = 0,
    Wall = 1,
}

impl Tile {
    pub fn render_text(&self) {
        match self {
            Tile::Empty => nodejs_helper::console::log("_|"),
            Tile::Wall => Wall::BlockWall.render_text(),
        }
    }

    pub fn render(&self) -> Vec<i32> {
        match self {
            Tile::Empty => vec![256 * 1 + 1],
            Tile::Wall => vec![256 * 1 + 1, 256 * 2 + 1],
        }
    }

    pub fn as_text(&self) -> String {
        let mut buf = String::from("");
        match self {
            Tile::Empty => buf.push_str("_|"),
            Tile::Wall => buf.push_str(Wall::BlockWall.as_text().as_str()),
        }
        buf
    }
    pub fn add_wall(&self) -> Result<Tile, ()> {
        match self {
            Tile::Empty => Ok(Tile::Wall),
            _ => Err(()),
        }
    }
}
