use crate::memory::Memory;
use crate::operations::*;

fn get_register_mut(m: &mut Memory, code: u8) -> &mut u8 {
    unsafe {
        match code {
            0b00000111 => &mut m.r.a,
            0 => &mut m.r.bc.ind.0,
            0b00000001 => &mut m.r.bc.ind.1,
            0b00000010 => &mut m.r.de.ind.0,
            0b00000011 => &mut m.r.de.ind.1,
            0b00000100 => &mut m.r.bc.ind.0,
            0b00000101 => &mut m.r.bc.ind.1,
            0b00000110 => m.mut_8(m.r.hl.comb),
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
            0b00000010 => m.r.de.ind.0,
            0b00000011 => m.r.de.ind.1,
            0b00000100 => m.r.hl.ind.0,
            0b00000101 => m.r.hl.ind.1,
            0b00000110 => m.read_8(m.r.hl.comb),
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
    m.r.pc += 1;
    let msb_2 = current_instruction & 0b11000000;
    let middle_3 = (current_instruction & 0b00111000) >> 3;
    let lsb_3 = current_instruction & 0b00000111;

    /*
        https://docs.rs/bitmatch/latest/bitmatch/
        https://users.rust-lang.org/t/why-is-a-lookup-table-faster-than-a-match-expression/24233
        https://archive.org/details/GameBoyProgManVer1.1/page/n95/mode/2up?view=theater
     */
    if msb_2 == 0 {
        if lsb_3 == 0b00000110 {
            // LD immediate
            let from_val = m.read_8(m.r.pc);
            m.r.pc += 1;
            let to = get_register_mut(m, middle_3);
            *to = from_val;
            cycles += 1;
            if middle_3 == 0b00000110 {
                cycles += 1;
            }
        }
    } else if msb_2 == 0b10000000 {
        if middle_3 == 0 {
            m.r.a = add_8(m.r.a, get_register_val(m, lsb_3), m);
        }
    } else if msb_2 == 0b01000000 {
        if middle_3 == 0b00000110 && lsb_3 == 0b00000110 {
            // HALT
            todo!();
        } else {
            // LD
            let from_val = get_register_val(m, lsb_3);
            let to = get_register_mut(m, middle_3);
            *to = from_val;
            if middle_3 == 0b00000110 || lsb_3 == 0b00000110 {
                cycles += 1;
            }
        }
    }

    cycles += 1;
}
