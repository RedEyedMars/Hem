#![feature(stmt_expr_attributes)]
#![feature(box_syntax, box_patterns)]
#![feature(ptr_to_from_bits)]
extern crate failure;
extern crate futures;
extern crate soa_derive;
//#[macro_use]
//extern crate auto_claw_render_gl_derive as render_gl_derive;

extern crate byteorder;
extern crate console_error_panic_hook;
extern crate generational_arena;
extern crate maplit;
extern crate packed_simd;
extern crate rand;

pub mod a;
pub mod g;
pub mod s;

macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}
pub(crate) use collection;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use packed_simd::{f32x4, m32x4};
pub fn replace_ps_f32x4(array: f32x4, value: f32, index: usize) -> Result<f32x4, failure::Error> {
    let mut filter = None;
    if index == 0 {
        filter = Some(m32x4::new(true, false, false, false));
    } else if index == 1 {
        filter = Some(m32x4::new(false, true, false, false));
    } else if index == 2 {
        filter = Some(m32x4::new(false, false, true, false));
    } else if index == 3 {
        filter = Some(m32x4::new(false, false, false, true));
    }

    if let Some(f) = filter {
        return Ok(f.select(f32x4::splat(value), array));
    } else {
        todo!();
    }
}

pub fn replace_ps_m32x4(array: m32x4, value: bool, index: usize) -> Result<m32x4, failure::Error> {
    let mut filter = None;
    if index == 0 {
        filter = Some(m32x4::new(true, false, false, false));
    } else if index == 1 {
        filter = Some(m32x4::new(false, true, false, false));
    } else if index == 2 {
        filter = Some(m32x4::new(false, false, true, false));
    } else if index == 3 {
        filter = Some(m32x4::new(false, false, false, true));
    }

    if let Some(f) = filter {
        return Ok(f.select(m32x4::splat(value), array));
    } else {
        todo!();
    }
}
