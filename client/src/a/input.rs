//use crate::a::b::Board;
use crate::a::e::b::AbilityClass;
use crate::a::{Direction, GameState};

use wasm_bindgen::prelude::*;

static mut INPUT_EVENTS: (Vec<InputEvent>, Vec<InputEvent>) = (vec![], vec![]);
static mut INPUT_TARGET: u8 = 0u8;

#[wasm_bindgen]
pub fn push_event(event: InputEvent) {
    unsafe {
        match INPUT_TARGET {
            0 => INPUT_EVENTS.0.push(event),
            1 => INPUT_EVENTS.1.push(event),
            _ => panic!("Unreachableable"),
        }
    }
}

pub fn detect_input(game: &mut GameState) -> Result<bool, failure::Error> {
    unsafe {
        // Swap
        INPUT_TARGET = match INPUT_TARGET {
            0 => 1,
            1 => 0,
            _ => panic!("unreachableljalk"),
        };
        // Process previous
        for event in match INPUT_TARGET {
            0 => &INPUT_EVENTS.1,
            1 => &INPUT_EVENTS.0,
            _ => panic!("Unreachableable"),
        }
        .iter()
        {
            event.handle_event(game)?
        }
        // Clear previous
        match INPUT_TARGET {
            0 => INPUT_EVENTS.1.clear(),
            1 => INPUT_EVENTS.0.clear(),
            _ => panic!("Unreachableable"),
        };

        handle_keys(game);
    }
    Ok(false)
}

unsafe fn handle_keys(game: &mut GameState) {
    for id in PRESSED_KEYS.iter() {
        match id {
            0 => {}
            65 => game.move_player(Direction::West),
            83 => game.move_player(Direction::South),
            68 => game.move_player(Direction::East),
            87 => game.move_player(Direction::North),
            69 => game.activate_player(AbilityClass::Normal),
            key => {
                nodejs_helper::console::log(format!("{}", key).as_str());
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct InputEvent {
    kind: u8,
    info0: u8,
    info1: f32,
    info2: f32,
}

#[wasm_bindgen]
impl InputEvent {
    pub fn key(pressed: u8) -> InputEvent {
        InputEvent {
            kind: 0u8,
            info0: pressed,
            info1: 0f32,
            info2: 0f32,
        }
    }
    pub fn unkey(pressed: u8) -> InputEvent {
        InputEvent {
            kind: 5u8,
            info0: pressed,
            info1: 0f32,
            info2: 0f32,
        }
    }
    pub fn hover(x: f32, y: f32) -> InputEvent {
        InputEvent {
            kind: 1u8,
            info0: 0u8,
            info1: x,
            info2: y,
        }
    }
    pub fn unhover(x: f32, y: f32) -> InputEvent {
        InputEvent {
            kind: 2u8,
            info0: 0u8,
            info1: x,
            info2: y,
        }
    }
    pub fn md(x: f32, y: f32) -> InputEvent {
        InputEvent {
            kind: 3u8,
            info0: 0u8,
            info1: x,
            info2: y,
        }
    }
    pub fn mu(x: f32, y: f32) -> InputEvent {
        InputEvent {
            kind: 4u8,
            info0: 0u8,
            info1: x,
            info2: y,
        }
    }
}

static mut PRESSED_KEYS: [u8; 16usize] = [0u8; 16usize];
impl InputEvent {
    pub fn kind(&self) -> u8 {
        self.kind
    }

    pub unsafe fn handle_event(&self, _game: &mut GameState) -> Result<(), failure::Error> {
        //game.has_changed = true;
        match self.kind {
            0 => {
                // key press
                if PRESSED_KEYS.iter().any(|id| *id == self.info0) {
                    return Ok(());
                } else {
                    let mut set_i = usize::MAX;
                    for (i, id) in PRESSED_KEYS.iter().enumerate() {
                        if *id == 0u8 {
                            set_i = i;
                            break;
                        }
                    }
                    if set_i < usize::MAX {
                        PRESSED_KEYS[set_i] = self.info0;
                    }
                }
            }
            5 => {
                // key unpress
                let mut set_i = usize::MAX;
                for (i, id) in PRESSED_KEYS.iter().enumerate() {
                    if *id == self.info0 {
                        set_i = i;
                        break;
                    }
                }
                if set_i < usize::MAX {
                    PRESSED_KEYS[set_i] = 0;
                }
            }
            1 => {
                //hover
                //game.hover_mouse(u32x2::new(event2 as u32, event3 as u32));
            }
            2 => {
                // exit
            }
            _ => { // meaningless
            }
        };
        Ok(())
    }
}
