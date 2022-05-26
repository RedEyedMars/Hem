use crate::a::e::b::{Ability, AbilityClass};
use crate::a::socket::SocketMessage;
use crate::a::Direction;
use packed_simd::f32x2;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum PlayerState {
    Idle,
}
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Player {
    state: PlayerState,
    xy: f32x2,
    vel: f32x2,
    equipped_abilties: Vec<Ability>,
    next_ability: Option<AbilityClass>,
}

const VEL_BOX: f32 = 0.1f32;
const STD_VEL: f32x2 = f32x2::new(0.05f32, 0.05f32);
impl Player {
    pub fn new(x: f32, y: f32) -> Player {
        Player {
            state: PlayerState::Idle,
            xy: f32x2::new(x, y),
            vel: f32x2::new(0f32, 0f32),
            equipped_abilties: Vec::new(),
            next_ability: None,
        }
    }
    pub fn new_enemy() -> Player {
        Player {
            state: PlayerState::Idle,
            xy: f32x2::new(0f32, 0f32),
            vel: f32x2::new(0f32, 0f32),
            equipped_abilties: Vec::new(),
            next_ability: None,
        }
    }
    pub fn get_x(&self) -> f32 {
        self.xy.extract(0)
    }
    pub fn get_y(&self) -> f32 {
        self.xy.extract(1)
    }

    pub fn execute(&mut self) {}
    pub fn render(&self) -> (i32, f32, f32) {
        match self.state {
            PlayerState::Idle => (256 * 16 + 1, self.xy.extract(0), self.xy.extract(1)),
        }
    }
    pub fn push(&mut self, d: Direction) {
        let new_vel = self.vel + d.vel() * STD_VEL;
        self.vel = f32x2::new(
            match new_vel.extract(0) {
                x if x > VEL_BOX => VEL_BOX,
                x if x < -VEL_BOX => -VEL_BOX,
                x => x,
            },
            match new_vel.extract(1) {
                y if y > VEL_BOX => VEL_BOX,
                y if y < -VEL_BOX => -VEL_BOX,
                y => y,
            },
        );
    }
    pub fn set_next_ability(&mut self, next_ability: AbilityClass) {
        self.next_ability = Some(next_ability);
    }

    pub fn get_socket_message(&self) -> SocketMessage {
        SocketMessage::Player(0u8, self.state.clone(), self.xy.clone(), self.vel.clone())
    }
    pub fn accept_state(&mut self, msg: SocketMessage) {
        //nodejs_helper::console::log(format!("Current:{:?}", self).as_str());
        //nodejs_helper::console::log(format!("Receive:{:?}", msg).as_str());
        match msg {
            SocketMessage::Player(_, state, xy, vel) => {
                self.state = state;
                self.xy = xy;
                self.vel = vel;
            }
            _ => panic!("Send message to player when was wrong to do so"),
        }
    }
}
