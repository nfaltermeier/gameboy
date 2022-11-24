extern crate bitflags;

mod memory;
mod opcodes;
mod operations;

use memory::Memory;

fn add(a: u8, b: u8) -> (u8, u8, u8, u8, u8) {
    let al = a & 0x0F;
    let bl = b & 0x0F;
    let (result, overflow) = a.overflowing_add(b);

    (
        result,
        (result == 0) as u8,
        ((al + bl) > 0x0F) as u8,
        0,
        overflow as u8,
    )
}

fn get_mut(m: &mut Memory, a: bool) -> &mut u8 {
    if a {
        &mut m.r.a
    } else {
        unsafe { &mut m.r.bc.ind.0 }
    }
}

fn main() {
    let mut m = Memory::default();
    // let a = get_mut(&mut m, true);
    // let b = get_mut(&mut m, false);
    // *a = 3;
    // *b = 7;
    println!("result: {:?}", add(0x3A, 0xC6));
    println!("result: {:?}", add(0x3C, 0xFF));
}
