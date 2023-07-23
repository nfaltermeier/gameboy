use bitmatch::bitmatch;

use crate::memory::Memory;
use crate::operations::*;

fn get_register_mut(mem: &mut Memory, code: u8) -> &mut u8 {
    match code {
        0b00000111 => &mut mem.r.a,
        0 => &mut mem.r.bc.ind.0,
        0b00000001 => &mut mem.r.bc.ind.1,
        0b00000010 => &mut mem.r.de.ind.0,
        0b00000011 => &mut mem.r.de.ind.1,
        0b00000100 => &mut mem.r.bc.ind.0,
        0b00000101 => &mut mem.r.bc.ind.1,
        0b00000110 => mem.mut_8(mem.r.hl.r16()),
        _ => panic!(
            "Unrecognized register code in get_register_mut: {:#b} (shifted to lsb?)",
            code
        ),
    }
}

fn get_register_val(mem: &Memory, code: u8) -> u8 {
    match code {
        0b00000111 => mem.r.a,
        0 => mem.r.bc.ind.0,
        0b00000001 => mem.r.bc.ind.1,
        0b00000010 => mem.r.de.ind.0,
        0b00000011 => mem.r.de.ind.1,
        0b00000100 => mem.r.hl.ind.0,
        0b00000101 => mem.r.hl.ind.1,
        0b00000110 => mem.read_8(mem.r.hl.r16()),
        _ => panic!(
            "Unrecognized register code in get_register_val: {:#b} (shifted to lsb?)",
            code
        ),
    }
}

fn u8s_to_u16(h: u8, l: u8) -> u16 {
    ((h as u16) << 8) | l as u16
}

fn u16_to_u8s(d: u16) -> (u8, u8) {
    ((d >> 8) as u8, d as u8)
}

#[bitmatch]
pub fn process_instruction(mem: &mut Memory) {
    let mut cycles = 0;
    let current_instruction = mem.read_8(mem.r.pc);
    mem.r.pc += 1;

    /*
       https://users.rust-lang.org/t/why-is-a-lookup-table-faster-than-a-match-expression/24233
       https://archive.org/details/GameBoyProgManVer1.1/page/n105/mode/2up?view=theater
    */
    #[bitmatch]
    match current_instruction {
        "00_lll_100" => {
            // INC r
            let to = get_register_mut(mem, l);
            mem.r.a = mem.read_8(mem.r.bc.r16());
            cycles += 1;
            todo!("finish implementing")
        }
        "00mmm110" => {
            // LD r n
            *get_register_mut(mem, m) = mem.read_8(mem.r.pc);
            mem.r.pc += 1;
            cycles += 1;
            if m == 0b00000110 {
                cycles += 1;
            }
        }
        "00_001_010" => {
            // LD A (BC)
            mem.r.a = mem.read_8(mem.r.bc.r16());
            cycles += 1;
        }
        "00_011_010" => {
            // LD A (DE)
            mem.r.a = mem.read_8(mem.r.de.r16());
            cycles += 1;
        }
        "00_101_010" => {
            // LD A (HLI)
            mem.r.a = mem.read_8(mem.r.hl.r16());
            mem.r.hl.uinc16();
            cycles += 1;
        }
        "00_111_010" => {
            // LD A (HLD)
            mem.r.a = mem.read_8(mem.r.hl.r16());
            mem.r.hl.udec16();
            cycles += 1;
        }
        "00_000_010" => {
            // LD (BC) A
            *mem.mut_8(mem.r.bc.r16()) = mem.r.a;
            cycles += 1;
        }
        "00_010_010" => {
            // LD (DE) A
            *mem.mut_8(mem.r.de.r16()) = mem.r.a;
            cycles += 1;
        }
        "00_100_010" => {
            // LD (HLI) A
            *mem.mut_8(mem.r.hl.r16()) = mem.r.a;
            mem.r.hl.uinc16();
            cycles += 1;
        }
        "00_110_010" => {
            // LD (HLD) A
            *mem.mut_8(mem.r.hl.r16()) = mem.r.a;
            mem.r.hl.udec16();
            cycles += 1;
        }
        "00_dd0_001" => {
            // LD dd nn
            let val = u8s_to_u16(mem.read_8(mem.r.pc + 1), mem.read_8(mem.r.pc));
            match d {
                0 => {
                    mem.r.bc.s16(val);
                }
                0b00000001 => {
                    mem.r.de.s16(val);
                }
                0b00000010 => {
                    mem.r.hl.s16(val);
                }
                0b00000011 => mem.r.sp = val,
                _ => panic!("Unknown load code in LD dd nn: {:#b}", d),
            };
            mem.r.pc += 2;
            cycles += 2;
        }
        "00_001_000" => {
            // LD (nn), SP
            let vals = u16_to_u8s(mem.r.sp);
            let addr = u8s_to_u16(mem.read_8(mem.r.pc + 1), mem.read_8(mem.r.pc));
            mem.write_8(addr, vals.0);
            mem.write_8(addr + 1, vals.1);
            mem.r.pc += 2;
            cycles += 4;
            todo!("Check if this actually works properly")
        }
        "01_110_110" => {
            // HALT
            todo!();
        }
        "01mmmlll" => {
            // LD r r'
            let from_val = get_register_val(mem, l);
            let to = get_register_mut(mem, m);
            *to = from_val;
            if m == 0b00000110 || l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_000lll" => {
            // ADD A, r and ADD A, (HL)
            mem.r.a = add_8(mem.r.a, get_register_val(mem, l), mem, false);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_001_lll" => {
            // ADC A, r and ADC A, (HL)
            mem.r.a = add_8(mem.r.a, get_register_val(mem, l), mem, true);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_010_lll" => {
            // SUB A, r and SUB A, (HL)
            mem.r.a = sub_8(mem.r.a, get_register_val(mem, l), mem, false);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_011_lll" => {
            // SBC A, r and SBC A, (HL)
            mem.r.a = sub_8(mem.r.a, get_register_val(mem, l), mem, true);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_100_lll" => {
            // AND A, r and AND A, (HL)
            mem.r.a = and_8(mem.r.a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_101_lll" => {
            // XOR A, r and XOR A, (HL)
            mem.r.a = xor_8(mem.r.a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_110_lll" => {
            // OR A, r and OR A, (HL)
            mem.r.a = or_8(mem.r.a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_111_lll" => {
            // CP A, r and CP A, (HL)
            cp_8(mem.r.a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "11_000_110" => {
            // ADD A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = add_8(mem.r.a, n, mem, false);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_001_110" => {
            // ADC A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = add_8(mem.r.a, n, mem, true);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_010_110" => {
            // SUB A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = sub_8(mem.r.a, n, mem, false);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_010_110" => {
            // SBC A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = sub_8(mem.r.a, n, mem, true);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_010_110" => {
            // AND A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = and_8(mem.r.a, n, mem);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_101_110" => {
            // XOR A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = xor_8(mem.r.a, n, mem);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_110_110" => {
            // OR A, n
            let n = mem.read_8(mem.r.pc);
            mem.r.a = or_8(mem.r.a, n, mem);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_111_110" => {
            // CP A, n
            let n = mem.read_8(mem.r.pc);
            cp_8(mem.r.a, n, mem);
            mem.r.pc += 1;
            cycles += 1;
        }
        "11_110_010" => {
            // LD A (C)
            mem.r.a = mem.read_8(0xFF00 + mem.r.bc.ind.1 as u16);
            cycles += 1;
        }
        "11_100_010" => {
            // LD (C) A
            *mem.mut_8(0xFF00 + mem.r.bc.ind.1 as u16) = mem.r.a;
            cycles += 1;
        }
        "11_110_000" => {
            // LD A (FF00 + n)
            mem.r.a = mem.read_8(0xFF00 + mem.r.pc);
            mem.r.pc += 1;
            cycles += 2;
        }
        "11_100_000" => {
            // LD A (FF00 + n)
            *mem.mut_8(0xFF00 + mem.r.pc) = mem.r.a;
            mem.r.pc += 1;
            cycles += 2;
        }
        "11_111_010" => {
            // LD A nn
            // TODO: check order of hi and lo
            let addr = u8s_to_u16(mem.read_8(mem.r.pc), mem.read_8(mem.r.pc + 1));
            mem.r.a = mem.read_8(addr);
            mem.r.pc += 2;
            cycles += 3;
        }
        "11_101_010" => {
            // LD nn A
            let addr = u8s_to_u16(mem.read_8(mem.r.pc), mem.read_8(mem.r.pc + 1));
            *mem.mut_8(addr) = mem.r.a;
            mem.r.pc += 2;
            cycles += 3;
        }
        "11_111_001" => {
            // LD SP HL
            mem.r.sp = mem.r.hl.r16();
            cycles += 1;
        }
        "11_qq0_101" => {
            // PUSH qq
            let vals = match q {
                0 => mem.r.bc.ind,
                0b00000001 => mem.r.de.ind,
                0b00000010 => mem.r.hl.ind,
                0b00000011 => (mem.r.a, mem.r.f.bits()),
                _ => panic!("Unknown register code in PUSH: {}", q),
            };
            mem.write_8(mem.r.sp - 1, vals.0);
            mem.write_8(mem.r.sp - 2, vals.1);
            mem.r.sp -= 2;
            cycles += 3;
        }
        "11_qq0_001" => {
            // POP qq
            let vals = (mem.read_8(mem.r.sp + 1), mem.read_8(mem.r.sp));
            if q == 0b00000011 {
                mem.r.a = vals.0;
                mem.r.set_flags_unchecked(vals.1);
            } else {
                let ptrs = match q {
                    0 => &mut mem.r.bc.ind,
                    0b00000001 => &mut mem.r.de.ind,
                    0b00000010 => &mut mem.r.hl.ind,
                    _ => panic!("Unknown register code in POP: {}", q),
                };
                *ptrs = vals;
            }
            mem.r.sp += 2;
            cycles += 2;
        }
        "11_111_000" => {
            // LDHL SP, e
            let e = mem.read_8(mem.r.pc) as i8;
            mem.r.pc += 1;
            let result = add_16_mixed(mem.r.sp, e, mem);
            mem.r.hl.s16(result);
            cycles += 2;
        }
        _ => todo!(),
    }

    cycles += 1;
}

#[cfg(test)]
mod tests {
    use crate::memory::Memory;

    use super::process_instruction;

    #[test]
    fn push_pop_same_val() {
        let mut m = Memory::default();
        let initial_bc = 0xDEAD;

        m.r.bc.s16(initial_bc);
        m.r.pc = 0x8000;
        m.r.sp = 0x9000;
        // PUSH bc
        m.write_8(0x8000, 0b11000101);
        // POP bc
        m.write_8(0x8001, 0b11000001);
        process_instruction(&mut m);
        m.r.bc.s16(0);
        process_instruction(&mut m);
        assert_eq!(
            m.r.bc.r16(),
            initial_bc,
            "PUSHing and then POPing changes the pushed value"
        );
    }

    #[test]
    fn pop_register_order() {
        let mut m = Memory::default();

        m.r.sp = 0xFFFC;
        m.write_8(0xFFFC, 0x5F);
        m.write_8(0xFFFD, 0x3C);

        m.r.pc = 0x8000;
        m.write_8(0x8000, 0b11_000_001);
        process_instruction(&mut m);

        assert_eq!(m.r.bc.ind.0, 0x3C);
        assert_eq!(m.r.bc.ind.1, 0x5F);
    }

    #[test]
    fn ld_16_byte_register_contents() {
        let mut m = Memory::default();
        m.r.pc = 0x8000;
        m.write_8(0x8000, 0b00_100_001);
        // Gameboy is little-endian, so least significant byte comes first
        m.write_8(0x8001, 0x5B);
        m.write_8(0x8002, 0x3A);
        process_instruction(&mut m);
        assert_eq!(m.r.hl.r16(), 0x3A5B);
        assert_eq!(m.r.hl.ind.0, 0x3A);
        assert_eq!(m.r.hl.ind.1, 0x5B);
    }
}
