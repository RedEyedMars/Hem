use crate::a::b::view::View;
use crate::a::b::{Adversary, AllyParty, AllyPartyStatus, AllyPartyStatusType, Faction, Guide};
use crate::collection;
use packed_simd::{f32x4, m32x4, mptrx4};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub struct ZoneAttribute {
    string: String,
    value: f32x4,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Zone {
    view: View,
    faction: Option<Faction>,
    guide: Vec<Guide>,
    ally: HashMap<String, RefCell<AllyParty>>,
    adversary: Vec<Adversary>,
    attribute: Vec<ZoneAttribute>,
    active_ally: HashMap<AllyPartyStatusType, Vec<AllyPartyStatus>>,
}

impl Zone {
    pub fn new(view: View, faction: Option<Faction>) -> Zone {
        Zone {
            view,
            faction,
            guide: Vec::new(),
            ally: HashMap::new(),
            adversary: Vec::new(),
            attribute: Vec::new(),
            active_ally: collection! [

                    AllyPartyStatusType::Working => Vec::new(),
                    AllyPartyStatusType::Researching => Vec::new(),
                    AllyPartyStatusType::Battling => Vec::new(),
                    AllyPartyStatusType::Resting => Vec::new(),
                    AllyPartyStatusType::Drinking => Vec::new(),
                    AllyPartyStatusType::Exploring => Vec::new(),
                    AllyPartyStatusType::Idle => Vec::new(),
            ],
        }
    }

    pub fn execute(&mut self) -> Result<(), failure::Error> {
        for _guide in self.guide.iter() {
            //self.execute_guide(guide, state)?;
        }
        for _adversary in self.adversary.iter() {
            //self.execute_adversary(adversary, state)?;
        }
        let mut completed_ally = Vec::new();
        for (_, ally_vec) in self.active_ally.iter_mut() {
            for ally in ally_vec {
                //self.execute_ally(ally, state)?;
                let mut slice = vec![
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                ];
                ally.execute()?.write_to_slice_unaligned(&mut slice);
                for (index, completed) in slice.into_iter().enumerate() {
                    unsafe {
                        if let Some(c) = completed.as_ref().unwrap().completed() {
                            completed_ally.push((
                                c.0,
                                c.1,
                                ally.refs.get_mut(index).unwrap().unwrap(),
                            ));
                        }
                    }
                }
            }
        }
        for ally in completed_ally.into_iter() {
            ally.0.complete(ally.1, ally.2, self)?;
        }
        Ok(())
    }
    pub fn add(&mut self, ally: RefCell<AllyParty>) {
        let mut names = [std::ptr::null(); 4usize];
        ally.borrow().name.write_to_slice_aligned(&mut names);
        for name in names {
            if !name.is_null() {
                self.ally.insert(*name, ally);
            }
        }
    }
    pub fn view(&self) -> View {
        self.view.clone()
    }
    /*
    pub fn remove(&mut self, name: &String) {
        let mut index = None;
        for (i, ally) in self.ally.iter().enumerate() {
            if ally.name.eq(name) {
                index = Some(i);
                break;
            }
        }
        if let Some(i) = index {
            //self.active_ally.remove(index);
        }
    }
    */
    pub unsafe fn deactivate(&mut self, ally: RefCell<AllyParty>) -> Result<(), failure::Error> {
        use crate::replace_ps_m32x4;
        let mut found_index = None;
        for (index, status_ally) in (*ally.borrow().status).refs.into_iter().enumerate() {
            if status_ally.unwrap() == ally {
                found_index = Some(index);
                break;
            }
        }
        if let Some(index) = found_index {
            {
                (*ally.borrow_mut().status).assigned_4 =
                    replace_ps_m32x4((*ally.borrow().status).assigned_4, false, index)?;
            }
            {
                (*ally.borrow_mut().status).progress_4.reset(index)?;
            }
            {
                ally.borrow_mut().status = std::ptr::null_mut();
            }
        }
        Ok(())
    }
    pub unsafe fn activate(&mut self, name: &str, start_state: AllyPartyStatusType) {
        use crate::replace_ps_m32x4;
        let name = String::from(name);
        if let Some(ally) = self.ally.get_mut(&name) {
            if !ally.borrow().status.is_null() && start_state.eq(&(*ally.borrow().status).kind) {
                return ();
            }

            let mut found_status = None;
            let ally_vec = self.active_ally.get_mut(&start_state).unwrap();
            for status in ally_vec.iter_mut() {
                if status.assigned_4.any() {
                    found_status = Some(status);
                }
            }
            if let Some(status) = found_status {
                ally.borrow_mut().status = status;
            } else {
                let status = AllyPartyStatus::new_with_ally(start_state, ally);
                ally_vec.push(status);
                if !ally.borrow().status.is_null() {
                    for (i, a) in (*ally.borrow_mut().status).refs.iter().enumerate() {
                        if let Some(al) = a {
                            if al == ally {
                                (*ally.borrow_mut().status).assigned_4 =
                                    replace_ps_m32x4((*ally.borrow().status).assigned_4, false, i)
                                        .expect("How could this fail");
                                break;
                            }
                        }
                    }
                }
                let last_index = ally_vec.len() - 1;
                ally.borrow_mut().status = ally_vec.get_mut(last_index).unwrap();
            }
        }
    }
}
