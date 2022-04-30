use crate::a::GameState;

use packed_simd::{f32x2, u32x2};

pub mod tiles;
use tiles::*;
use wasm_bindgen::prelude::*;

#[repr(u8)]
#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Wall {
    BlockWall,
}

impl Wall {
    pub fn render_text(&self) {
        nodejs_helper::console::log("W|")
    }

    pub fn as_text(&self) -> String {
        String::from("W|")
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Board {
    tiles: Vec<Tile>,
    name: Vec<u8>,
    width: u32,
    height: u32,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            tiles: Vec::new(),
            name: Vec::new(),
            width: 10,
            height: 6,
        }
    }
    pub fn new(name: String, width: usize, height: usize) -> Board {
        let mut tiles = Vec::with_capacity(width * height);
        for _ in 0..height {
            for _ in 0..width {
                tiles.push(Tile::Empty);
            }
        }
        Board {
            /*
            grid_img: Img::new(
                "bot_1.png".to_string(),
                0f32,
                SizeMode::Bot,
                Animation::MainXShift2x16,
                res,
            )?,*/
            tiles: tiles,
            name: name.into_bytes().to_vec(),
            width: width as u32,
            height: height as u32,
        }
    }
    pub fn execute(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }
    pub fn render(&self, _game: &GameState) -> Result<(), failure::Error> {
        //self.grid_img.render(game);
        Ok(())
    }
    pub fn render_text(&self) {
        for y in 0..self.height as u32 {
            let mut buf = String::from("");
            for x in 0..self.width as u32 {
                buf.push_str(
                    self.tiles[self.index(u32x2::new(x, (self.height - 1) as u32 - y))]
                        .as_text()
                        .as_str(),
                );
            }
            nodejs_helper::console::log(buf.as_str());
        }
        nodejs_helper::console::log("===================================================");
    }
    pub fn as_text(&self) -> String {
        let mut buf = String::from("");
        for y in 0..self.height as u32 {
            for x in 0..self.width as u32 {
                buf.push_str(
                    self.tiles
                        .get(self.index(u32x2::new(x, (self.height - 1) as u32 - y)))
                        .unwrap()
                        .as_text()
                        .as_str(),
                );
            }
            buf.push_str("\n");
        }
        buf.push_str("===================================================");
        buf
    }
    pub fn walls(&self) -> Vec<(Wall, f32x2)> {
        let mut result = Vec::new();
        for (index, tile) in self.tiles.iter().enumerate() {
            if let Tile::Wall = tile {
                result.push((Wall::BlockWall, self.fxy(index)));
            }
        }
        result
    }
    pub fn add_wall(&mut self, xy: u32x2) -> Result<(), ()> {
        let index = ((xy.extract(1)) * self.width + (xy.extract(0))) as usize;
        self.tiles.remove(index);
        self.tiles.insert(index, Tile::Wall);
        Ok(())
    }
    pub fn get_tile(&self, x: u32, y: u32) -> &Tile {
        self.tiles.get(self.index(u32x2::new(x, y))).unwrap()
    }
    pub fn get_tile_from_position(&self, xy: u32x2) -> &Tile {
        self.tiles.get(self.index(xy)).unwrap()
    }

    pub fn index(&self, xy: u32x2) -> usize {
        ((xy.extract(1)) * self.width + (xy.extract(0))) as usize
    }
    pub fn xy(&self, i: usize) -> u32x2 {
        u32x2::new(i as u32 % self.width as u32, i as u32 / self.width as u32)
    }
    pub fn fxy(&self, i: usize) -> f32x2 {
        f32x2::new(
            (i as u32 % self.width as u32) as f32,
            (i as u32 / self.width as u32) as f32,
        )
    }
    pub fn apply_on_tile<T>(
        &mut self,
        obj: &T,
        xy: u32x2,
        f: &dyn Fn(&Tile, &T) -> Result<Tile, ()>,
    ) -> Result<Tile, ()>
    where
        T: std::any::Any,
    {
        use std::mem::replace;
        let index = ((xy.extract(1)) * self.width + (xy.extract(0))) as usize;
        let tile = self.tiles.get(index).unwrap().clone();
        let v = f(&tile, obj)?;
        let _ = replace(&mut self.tiles[index], v.clone());
        Ok(v)
    }

    pub fn apply_on_both_tile<T>(
        &mut self,
        obj: &T,
        xy1: u32x2,
        f1: &dyn Fn(&Tile, &T) -> Result<Tile, ()>,
        xy2: u32x2,
        f2: &dyn Fn(&Tile, &T) -> Result<Tile, ()>,
    ) -> Result<(Tile, Tile), ()>
    where
        T: std::any::Any,
    {
        use std::mem::replace;
        let index1 = ((xy1.extract(1)) * self.width + (xy1.extract(0))) as usize;
        let index2 = ((xy2.extract(1)) * self.width + (xy2.extract(0))) as usize;
        let tile1 = self.tiles.get(index1).unwrap().clone();
        let tile2 = self.tiles.get(index2).unwrap().clone();
        let v = (f1(&tile1, obj)?, f2(&tile2, obj)?);
        let _ = replace(&mut self.tiles[index1], v.0.clone());
        let _ = replace(&mut self.tiles[index2], v.1.clone());
        Ok(v)
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn name(&self) -> &Vec<u8> {
        &self.name
    }
}
