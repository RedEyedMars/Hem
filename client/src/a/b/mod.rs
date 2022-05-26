use crate::a::GameState;

use packed_simd::{cptrx4, m32x4, mptrx4};

use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub mod ally;
pub mod progress;
pub mod view;
pub mod zone;

use progress::{Progress4, ProgressAttribute};
use view::View;
use zone::Zone;

#[repr(u8)]
#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Faction {
    Might,
    Magic,
    Guile,
}

#[repr(u8)]
#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Guild {
    Warrior,
    Barbarian,
    Guardian,
    //
    Priest,
    WitchDoctor,
    Herbalist,
    //
    Rogue,
    Hunter,
    Assasin,
    //
    Mage,
    Summoner,
    Elementalist,
}

#[repr(u8)]
#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ExecutionErrorType {
    ExecutionBaseLoop,
}

#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ExecutionError {
    value: String,
    kind: ExecutionErrorType,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum AllyPartyStatusType {
    Working,
    Researching,
    Battling,
    Resting,
    Drinking,
    Exploring,
    Idle,
}

#[derive(PartialEq, Clone, Debug)]
pub struct AllyPartyStatus {
    kind: AllyPartyStatusType,
    assigned_4: m32x4,
    refs: [Option<RefCell<AllyParty>>; 4usize],
    progress_4: Progress4,
    progress_attributes: ProgressAttribute,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Debug)]
pub enum PartyType {
    Player,
}

#[derive(PartialEq, Clone, Debug)]
pub struct AllyParty {
    kind: PartyType,
    status: *mut AllyPartyStatus,
    name: cptrx4<String>,
    faction: cptrx4<Faction>,
    guild: cptrx4<Guild>,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Debug)]
pub enum AdversaryType {
    Enemy,
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Debug)]
pub struct Adversary {
    name: String,
    kind: AdversaryType,
    faction: Faction,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Debug)]
pub enum GuideType {
    Researcher,
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Debug)]
pub struct Guide {
    value: String,
    kind: GuideType,
}

#[repr(u8)]
#[derive(PartialEq, Clone, Debug)]
pub enum ProgressStateType {
    Idle,
    Processing,
    Complete,
    Void,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ProgressState<T> {
    kind: ProgressStateType,
    state: T,
    value: f32,
}

impl<T: Clone> ProgressState<T> {
    pub fn completed(&self) -> Option<(T, f32)> {
        if let ProgressStateType::Complete = self.kind {
            return Some((self.state.clone(), self.value));
        } else {
            return None;
        }
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Debug)]
pub struct Board {
    current_view: View,
    zone: HashMap<View, Zone>,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            current_view: View::Town,
            zone: HashMap::new(),
        }
    }
    pub fn new(view: View) -> Board {
        Board {
            /*
            grid_img: Img::new(
                "bot_1.png".to_string(),
                0f32,
                SizeMode::Bot,
                Animation::MainXShift2x16,
                res,
            )?,*/
            current_view: view,
            zone: HashMap::new(),
        }
    }
    pub fn execute(&mut self) -> Result<(), failure::Error> {
        for (_view, zone) in self.zone.iter_mut() {
            zone.execute()?;
        }
        Ok(())
    }
    pub fn render(&self, _game: &GameState) -> Result<(), failure::Error> {
        //self.grid_img.render(game);
        Ok(())
    }
    pub unsafe fn render_text(&self) {
        self.current_view.render_text();
        nodejs_helper::console::log("===================================================");
    }
    pub fn as_text(&self) -> String {
        let mut buf = String::from("");
        buf.push_str(self.current_view.as_text());
        buf.push_str("===================================================");
        buf
    }
    pub fn view(&self) -> &View {
        &self.current_view
    }
    pub fn change_view(&mut self, new_view: View) {
        self.current_view = new_view;
    }
    pub fn add_zone(&mut self, zone: Zone) {
        self.zone.insert(zone.view(), zone);
    }
}
//test
impl Board {
    pub fn test_init_1(&mut self) {
        let mut zone = Zone::new(View::Town, None);
        unsafe {
            zone.add(RefCell::new(AllyParty::new(
                String::from("joe"),
                Faction::Might,
                Guild::Warrior,
            )));
            zone.activate("joe", AllyPartyStatusType::Resting);
        }
        self.add_zone(zone);
    }
}
