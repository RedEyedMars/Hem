use packed_simd::{m16x4, u16x4};
use serde::Serialize;
use wasm_bindgen::prelude::*;

static mut XS: [u16x4; 256usize] = [u16x4::splat(0); 256];

static mut X_PTR: *mut u16x4 = unsafe { XS.as_mut_ptr() };

//screen
#[derive(Clone, Debug)]
pub struct Screen {
    layer: [Layer; 4usize],
}
#[derive(Clone, Debug)]
pub enum CycleState {
    Bounce,
    Cycle,
    Flash, //Final frame is held
    Blink, //Final frame is held and the texture is hidden
    Idle,
}
#[derive(Clone, Debug)]
pub struct Element {
    s: u32, // count of elements contained
    t: u32, // texture index in the t vec
    x: *mut u16x4,
    y: Vec<u16x4>,
    w: Vec<u16x4>,
    h: Vec<u16x4>,
    v: Vec<m16x4>, // whether a texture is visable
}

#[derive(Serialize)]
pub struct ElementDto<'a> {
    s: &'a u32,
    t: &'a u32, // texture index in the t vec
    x: usize,   /*
                y: &'a [u16x4],
                w: &'a [u16x4],
                h: &'a [u16x4],
                v: &'a [m16x4], // whether a texture is visable
                */
}

impl Element {
    /*
    pub fn execute(&mut self, dynamic: &mut Option<Dynamic>) {
        if let Some(dynamic) = dynamic {
            self.x = self
                .x
                .iter()
                .enumerate()
                .map(|(i, x)| *x + u16x4::from_slice_aligned(dynamic.dx.get(i).unwrap()))
                .collect();
            self.y = self
                .y
                .iter()
                .enumerate()
                .map(|(i, y)| *y + u16x4::from_slice_aligned(dynamic.dy.get(i).unwrap()))
                .collect();

            for i in 0..self.x.len() {
                dynamic.dx[i] = [0u16; 4usize];
                dynamic.dy[i] = [0u16; 4usize];
            }
        }
    }*/
}

#[derive(Clone, Debug)]
pub struct Dynamic {
    t: Vec<u16>, // available textures for this element class,
    c: CycleState,
    dx: Vec<[u16; 4usize]>,
    dy: Vec<[u16; 4usize]>,
}

impl Dynamic {
    pub fn mv(&mut self, index: usize, dx: u16, dy: u16) {
        self.dx.get_mut(index / 16).unwrap()[index % 16] += dx;
        self.dy.get_mut(index / 16).unwrap()[index % 16] += dy;
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    element: Vec<Element>,
    dynamic: Vec<Option<Dynamic>>,
}

impl Screen {
    pub fn new() -> Screen {
        let mut first = None;
        unsafe {
            first = Some(X_PTR.clone());
            X_PTR = X_PTR.add(1);
        }
        Screen {
            layer: [
                Layer {
                    element: vec![Element {
                        s: 1,
                        t: 1,
                        x: first.unwrap(),
                        y: vec![u16x4::splat(2)],
                        w: vec![u16x4::splat(2)],
                        h: vec![u16x4::splat(2)],
                        v: vec![m16x4::splat(true)],
                    }],
                    dynamic: vec![],
                },
                Layer {
                    element: vec![],
                    dynamic: vec![],
                },
                Layer {
                    element: vec![],
                    dynamic: vec![],
                },
                Layer {
                    element: vec![],
                    dynamic: vec![],
                },
            ],
        }
    }
}

impl Screen {
    pub fn elements(&mut self, index: usize) -> Vec<JsValue> {
        unsafe {
            *self.layer[index].element.get_mut(0).unwrap().x =
                (*self.layer[index].element.get_mut(0).unwrap().x) + u16x4::splat(2);
            self.layer[index]
                .element
                .iter()
                .map(|e| {
                    JsValue::from_serde(&ElementDto {
                        s: &e.s,
                        t: &e.t,
                        x: e.x.to_bits(), /*
                                          y: e.y.as_slice(),
                                          w: e.w.as_slice(),
                                          h: e.h.as_slice(),
                                          v: e.v.as_slice(),*/
                    })
                    .unwrap()
                })
                .collect()
        }
    }
}
