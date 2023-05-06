extern crate bitflags;
extern crate bitmatch;

mod memory;
mod opcodes;
mod operations;

use memory::Memory;

use crate::opcodes::process_instruction;

fn main() {
    let mut m = Memory::default();
    m.r.set_flags_unchecked(0xFF);
    println!("result: {:#b}", m.r.f.bits());
    println!("result: {}", (-1) >> 1);
    println!("result: {}", 0xFFu8 >> 1);
    println!("result: {:?}", 1_u16.overflowing_sub(2));

    m.r.pc = 0;
    m.write_8(0, 0b00_001_010);
    process_instruction(&mut m);
}
