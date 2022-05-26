use wasm_bindgen::prelude::*;

#[repr(u8)]
#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum View {
    Town,
    Tavern,
    Battle,
    Guild,
    Research,
}

impl View {
    pub unsafe fn render_text(&self) -> () {
        match self {
            View::Town => nodejs_helper::console::log("Town overview"),
            View::Tavern => nodejs_helper::console::log("Tavern"),
            View::Battle => nodejs_helper::console::log("Battle"),
            View::Guild => nodejs_helper::console::log("Guild"),
            View::Research => nodejs_helper::console::log("Research"),
        }
    }

    pub fn as_text(&self) -> &str {
        match self {
            View::Town => "Town overview",
            View::Tavern => "Tavern",
            View::Battle => "Battle",
            View::Guild => "Guild",
            View::Research => "Research",
        }
    }
    pub fn from(name: &String) -> View {
        match name.as_ref() {
            "T" => View::Town,
            "A" => View::Tavern,
            "B" => View::Battle,
            "G" => View::Guild,
            "R" => View::Research,
            _ => todo!(),
        }
    }
}
