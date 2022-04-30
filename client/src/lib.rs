#![feature(stmt_expr_attributes)]
#![feature(box_syntax, box_patterns)]
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

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
