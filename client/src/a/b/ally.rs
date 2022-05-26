use crate::a::b::progress::{Progress4, ProgressAttribute};
use crate::a::b::zone::Zone;
use crate::a::b::{
    AllyParty, AllyPartyStatus, AllyPartyStatusType, Faction, Guild, PartyType, ProgressState,
    ProgressStateType,
};
use packed_simd::{cptrx4, m32x4, mptrx4};
use std::cell::RefCell;

impl AllyParty {
    pub fn new(name: String, faction: Faction, guild: Guild) -> AllyParty {
        AllyParty {
            name: cptrx4::new(
                &name.clone(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            ),
            kind: PartyType::Player,
            faction: cptrx4::new(
                &faction.clone(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            ),
            guild: cptrx4::new(
                &guild.clone(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            ),
            status: std::ptr::null_mut(),
        }
    }
}

//Ally
impl AllyPartyStatus {
    pub fn new(kind: AllyPartyStatusType) -> AllyPartyStatus {
        AllyPartyStatus {
            kind,
            assigned_4: m32x4::splat(false),
            refs: [None, None, None, None],
            progress_4: Progress4::new(),
            progress_attributes: ProgressAttribute::new(),
        }
    }
    pub fn new_with_ally(
        kind: AllyPartyStatusType,
        first: &mut RefCell<AllyParty>,
    ) -> AllyPartyStatus {
        AllyPartyStatus {
            kind,
            assigned_4: m32x4::new(true, false, false, false),
            refs: [Some(first.clone()), None, None, None],
            progress_4: Progress4::new(),
            progress_attributes: ProgressAttribute::new(),
        }
    }

    pub fn active(&self) -> bool {
        match self.kind.clone() {
            AllyPartyStatusType::Idle => false,
            _ => true,
        }
    }

    pub fn execute(
        &mut self,
    ) -> Result<cptrx4<ProgressState<AllyPartyStatusType>>, failure::Error> {
        let percentage = self.progress_4.percentage(&self.progress_attributes);
        if self.active() {
            let allies_assigned = self.assigned_4;
            let can_progress =
                self.progress_4.can_progress(&self.progress_attributes) & allies_assigned;
            let values = self
                .progress_4
                .completion_percentage(&self.progress_attributes);

            let result = can_progress.select(
                cptrx4::new(
                    &ProgressState::<AllyPartyStatusType> {
                        kind: ProgressStateType::Processing,
                        state: self.kind.clone(),
                        value: percentage.extract(0),
                    },
                    &ProgressState::<AllyPartyStatusType> {
                        kind: ProgressStateType::Processing,
                        state: self.kind.clone(),
                        value: percentage.extract(1),
                    },
                    &ProgressState::<AllyPartyStatusType> {
                        kind: ProgressStateType::Processing,
                        state: self.kind.clone(),
                        value: percentage.extract(2),
                    },
                    &ProgressState::<AllyPartyStatusType> {
                        kind: ProgressStateType::Processing,
                        state: self.kind.clone(),
                        value: percentage.extract(3),
                    },
                ),
                allies_assigned.select(
                    cptrx4::new(
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Complete,
                            state: self.kind.clone(),
                            value: values.extract(0),
                        },
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Complete,
                            state: self.kind.clone(),
                            value: values.extract(1),
                        },
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Complete,
                            state: self.kind.clone(),
                            value: values.extract(2),
                        },
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Complete,
                            state: self.kind.clone(),
                            value: values.extract(3),
                        },
                    ),
                    cptrx4::new(
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Void,
                            state: self.kind.clone(),
                            value: 0f32,
                        },
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Void,
                            state: self.kind.clone(),
                            value: 0f32,
                        },
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Void,
                            state: self.kind.clone(),
                            value: 0f32,
                        },
                        &ProgressState::<AllyPartyStatusType> {
                            kind: ProgressStateType::Void,
                            state: self.kind.clone(),
                            value: 0f32,
                        },
                    ),
                ),
            );
            self.progress_4
                .progress(can_progress, &self.progress_attributes);
            return Ok(result);
        } else {
            return Ok(cptrx4::new(
                &ProgressState::<AllyPartyStatusType> {
                    kind: ProgressStateType::Idle,
                    state: self.kind.clone(),
                    value: percentage.extract(0),
                },
                &ProgressState::<AllyPartyStatusType> {
                    kind: ProgressStateType::Idle,
                    state: self.kind.clone(),
                    value: percentage.extract(1),
                },
                &ProgressState::<AllyPartyStatusType> {
                    kind: ProgressStateType::Idle,
                    state: self.kind.clone(),
                    value: percentage.extract(2),
                },
                &ProgressState::<AllyPartyStatusType> {
                    kind: ProgressStateType::Idle,
                    state: self.kind.clone(),
                    value: percentage.extract(3),
                },
            ));
        }
    }
}

impl AllyPartyStatusType {
    pub fn complete(
        &self,
        _value: f32,
        ally: RefCell<AllyParty>,
        zone: &mut Zone,
    ) -> Result<(), failure::Error> {
        unsafe {
            zone.deactivate(ally)?;
        }
        Ok(())
    }
}
