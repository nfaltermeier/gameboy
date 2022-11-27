extern crate bitflags;
extern crate bitmatch;

mod memory;
mod opcodes;
mod operations;

use memory::Memory;

fn main() {
    let mut m = Memory::default();
    m.r.set_flags_unchecked(0xFF);
    println!("result: {:#b}", m.r.f.bits());
    println!("result: {}", (-1) >> 1);
    println!("result: {}", 0xFFu8 >> 1);
    println!("result: {:?}", 1_u16.overflowing_sub(2));
}
