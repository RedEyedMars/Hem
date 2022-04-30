use packed_simd::f32x2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
pub enum MonsterState {
    Idle,
}
#[wasm_bindgen]
pub struct Monster {
    state: MonsterState,
    xy: f32x2,
}

impl Monster {
    pub fn get_x(&self) -> f32 {
        self.xy.extract(0)
    }
    pub fn get_y(&self) -> f32 {
        self.xy.extract(1)
    }
    pub fn get_render(&self) -> (&str, u16) {
        match self.state {
            MonsterState::Idle => ("player", 1),
        }
    }
}
