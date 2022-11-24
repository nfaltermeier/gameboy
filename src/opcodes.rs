use crate::memory::Memory;
use crate::operations::*;

fn get_register_mut(m: &mut Memory, code: u8) -> &mut u8 {
    unsafe {
        match code {
            0b00000111 => &mut m.r.a,
            0 => &mut m.r.bc.ind.0,
            0b00000001 => &mut m.r.bc.ind.1,
            0b00000011 => &mut m.r.de.ind.0,
            0b00000100 => &mut m.r.de.ind.1,
            0b00000101 => &mut m.r.bc.ind.0,
            0b00000110 => &mut m.r.bc.ind.1,
            _ => panic!(
                "Unrecognized register code in get_register_mut: {:#b} (shifted to lsb?)",
                code
            ),
        }
    }
}

fn get_register_val(m: &Memory, code: u8) -> u8 {
    unsafe {
        match code {
            0b00000111 => m.r.a,
            0 => m.r.bc.ind.0,
            0b00000001 => m.r.bc.ind.1,
            0b00000011 => m.r.de.ind.0,
            0b00000100 => m.r.de.ind.1,
            0b00000101 => m.r.bc.ind.0,
            0b00000110 => m.r.bc.ind.1,
            _ => panic!(
                "Unrecognized register code in get_register_val: {:#b} (shifted to lsb?)",
                code
            ),
        }
    }
}

pub fn process_instruction(m: &mut Memory) {
    let mut cycles = 0;
    let current_instruction = m.read_8(m.r.pc);
    let msb_2 = current_instruction & 0b11000000;
    let middle_3 = current_instruction & 0b00111000;
    let lsb_3 = current_instruction & 0b00000111;

    if msb_2 == 0b10000000 {
        if middle_3 == 0 {
            m.r.a = add_8(m.r.a, get_register_val(m, lsb_3), m);
            cycles += 1;
        }
    }

    m.r.pc += 1;
}
