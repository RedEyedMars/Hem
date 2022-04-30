#[derive(Clone, Debug)]
pub enum AbilityClass {
    Normal,
    Special,
}

#[derive(Clone, Debug)]
pub enum Ability {
    Slash,
}

impl Ability {
    pub fn cl(&self) -> AbilityClass {
        match self {
            Ability::Slash => AbilityClass::Normal,
        }
    }
}
