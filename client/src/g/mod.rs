use packed_simd::u32x2;
use serde::Serialize;
use std::collections::HashMap;

pub mod screen;

#[derive(Serialize)]
pub struct GraphicPositions {
    pub xys: HashMap<u16, Vec<WrapperU32x2>>,
    //pub ys: HashMap<u16, Vec<u32>>,
}
#[derive(Serialize)]
pub struct GraphicLayer {
    pub left: GraphicPositions,
    pub right: GraphicPositions,
}

use serde::ser::{SerializeSeq, Serializer};
pub struct WrapperU32x2(u32x2);
impl From<u32x2> for WrapperU32x2 {
    fn from(w: u32x2) -> WrapperU32x2 {
        WrapperU32x2(w)
    }
}
impl Serialize for WrapperU32x2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.0.extract(0))?;
        seq.serialize_element(&self.0.extract(1))?;
        seq.end()
    }
}
