use bitmatch::bitmatch;

use crate::memory::{MemoryController, RegisterFlags};
use crate::operations::*;

fn get_register_mut(mem: &mut dyn MemoryController, code: u8) -> &mut u8 {
    match code {
        0b00000111 => &mut mem.r().a,
        0 => &mut mem.r().bc.ind.0,
        0b00000001 => &mut mem.r().bc.ind.1,
        0b00000010 => &mut mem.r().de.ind.0,
        0b00000011 => &mut mem.r().de.ind.1,
        0b00000100 => &mut mem.r().bc.ind.0,
        0b00000101 => &mut mem.r().bc.ind.1,
        0b00000110 => mem.mut_8(mem.r_i().hl.r16()),
        _ => panic!(
            "Unrecognized register code in get_register_mut: {:#b} (shifted to lsb?)",
            code
        ),
    }
}

fn get_register_val(mem: &dyn MemoryController, code: u8) -> u8 {
    match code {
        0b00000111 => mem.r_i().a,
        0 => mem.r_i().bc.ind.0,
        0b00000001 => mem.r_i().bc.ind.1,
        0b00000010 => mem.r_i().de.ind.0,
        0b00000011 => mem.r_i().de.ind.1,
        0b00000100 => mem.r_i().hl.ind.0,
        0b00000101 => mem.r_i().hl.ind.1,
        0b00000110 => mem.read_8(mem.r_i().hl.r16()),
        _ => panic!(
            "Unrecognized register code in get_register_val: {:#b} (shifted to lsb?)",
            code
        ),
    }
}

fn get_register_pair_val(mem: &dyn MemoryController, code: u8) -> u16 {
    match code {
        0 => mem.r_i().bc.r16(),
        0b00000001 => mem.r_i().de.r16(),
        0b00000010 => mem.r_i().hl.r16(),
        0b00000011 => mem.r_i().sp,
        _ => panic!(
            "Unrecognized register code in get_register_val: {:#b} (shifted to lsb?)",
            code
        ),
    }
}

pub fn u8s_to_u16(h: u8, l: u8) -> u16 {
    let val =  ((h as u16) << 8) | l as u16;
    val
}

pub fn u16_to_u8s(d: u16) -> (u8, u8) {
    ((d >> 8) as u8, d as u8)
}

fn check_jump_condition(cc: u8, mem: &dyn MemoryController) -> bool {
    (cc == 0 && !mem.r_i().f.contains(RegisterFlags::Z))
        || (cc == 0b00000001 && mem.r_i().f.contains(RegisterFlags::Z))
        || (cc == 0b00000010 && !mem.r_i().f.contains(RegisterFlags::CY))
        || (cc == 0b00000011 && mem.r_i().f.contains(RegisterFlags::CY))
}

#[bitmatch]
pub fn process_instruction(mem: &mut dyn MemoryController) -> u64 {
    let mut cycles = 0;
    println!("pc: {:#x}", mem.r_i().pc);
    let current_instruction = mem.read_8(mem.r_i().pc);
    println!("ins: {:#b}", current_instruction);
    mem.r().pc += 1;

    /*
       https://users.rust-lang.org/t/why-is-a-lookup-table-faster-than-a-match-expression/24233
       https://archive.org/details/GameBoyProgManVer1.1/page/n123/mode/2up?view=theater
    */
    #[bitmatch]
    match current_instruction {
        "00_000_000" => {
            // NOP
        }
        "00_000_111" => {
            // RLCA
            mem.r().a = rlc(mem.r().a, mem, true);
        }
        "00_001_010" => {
            // LD A (BC)
            mem.r().a = mem.read_8(mem.r_i().bc.r16());
            cycles += 1;
        }
        "00_010_111" => {
            // RLA
            mem.r().a = rl(mem.r().a, mem, true);
        }
        "00_001_111" => {
            // RRCA
            mem.r().a = rrc(mem.r().a, mem, true);
        }
        "00_011_000" => {
            // JR e
            let e = mem.read_8(mem.r_i().pc) as i8;
            let result = mem.r().pc.checked_add_signed(e.into());
            match result {
                None => panic!("JR over/underflowed, not sure how this should behave"),
                Some(v) => {
                    mem.r().pc = v + 1;
                }
            }
            cycles += 2;
            todo!("Check if this actually works properly")
        }
        "00_011_010" => {
            // LD A (DE)
            mem.r().a = mem.read_8(mem.r_i().de.r16());
            cycles += 1;
        }
        "00_011_111" => {
            // RRA
            mem.r().a = rr(mem.r().a, mem, true);
        }
        "00_100_111" => {
            // DAA
            // done according to https://forums.nesdev.org/viewtopic.php?t=15944
            if !mem.r().f.contains(RegisterFlags::N) {
                // Addition
                if mem.r().f.contains(RegisterFlags::CY) || mem.r().a > 0x99 {
                    mem.r().a += 0x60;
                    mem.r().f.set(RegisterFlags::CY, true);
                }

                if mem.r().f.contains(RegisterFlags::H) || (mem.r().a & 0x0F) > 0x09 {
                    mem.r().a += 0x06;
                }
            } else {
                // Subtraction
                if mem.r().f.contains(RegisterFlags::CY) {
                    mem.r().a -= 0x60;
                }

                if mem.r().f.contains(RegisterFlags::H) {
                    mem.r().a -= 0x06;
                }
            }

            let a = mem.r_i().a;
            mem.r().f.set(RegisterFlags::Z, a == 0);
            mem.r().f.set(RegisterFlags::H, false);
        }
        "00_101_111" => {
            // CPL
            mem.r().a = !mem.r().a;
            mem.r().f.set(RegisterFlags::H, true);
            mem.r().f.set(RegisterFlags::N, true);
        }
        "00_1cc_000" => {
            // JR cc, e
            if check_jump_condition(c, mem) {
                let e = mem.read_8(mem.r_i().pc) as i8;
                let result = mem.r().pc.checked_add_signed(e.into());
                match result {
                    None => panic!("JR over/underflowed, not sure how this should behave"),
                    Some(v) => {
                        mem.r().pc = v + 1;
                    }
                }
                cycles += 2;
            } else {
                mem.r().pc += 1;
                cycles += 1;
            }
        }
        "00_lll_100" => {
            // INC r, INC (HL)
            let val = inc_8(get_register_val(mem, l), mem);
            *get_register_mut(mem, l) = val;
            if l == 0b00000110 {
                cycles += 2;
            }
        }
        "00_lll_101" => {
            // DEC r, DEC (HL)
            let val = dec_8(get_register_val(mem, l), mem);
            *get_register_mut(mem, l) = val;
            if l == 0b00000110 {
                cycles += 2;
            }
        }
        "00_mmm_110" => {
            // LD r n
            *get_register_mut(mem, m) = mem.read_8(mem.r_i().pc);
            mem.r().pc += 1;
            cycles += 1;
            if m == 0b00000110 {
                cycles += 1;
            }
        }
        "00_ss1_001" => {
            // ADD HL, (ss)
            let val = add_16(mem.r().hl.r16(), get_register_pair_val(mem, s), mem);
            mem.r().hl.s16(val);
            cycles += 1;
        }
        "00_ss0_011" => {
            // INC ss
            let val = get_register_pair_val(mem, s);
            mem.r().hl.s16(inc_16(val));
            cycles += 1;
        }
        "00_ss1_011" => {
            // DEC ss
            let val = get_register_pair_val(mem, s);
            mem.r().hl.s16(dec_16(val));
            cycles += 1;
        }
        "00_101_010" => {
            // LD A (HLI)
            mem.r().a = mem.read_8(mem.r_i().hl.r16());
            mem.r().hl.uinc16();
            cycles += 1;
        }
        "00_111_010" => {
            // LD A (HLD)
            mem.r().a = mem.read_8(mem.r_i().hl.r16());
            mem.r().hl.udec16();
            cycles += 1;
        }
        "00_000_010" => {
            // LD (BC) A
            *mem.mut_8(mem.r_i().bc.r16()) = mem.r().a;
            cycles += 1;
        }
        "00_010_010" => {
            // LD (DE) A
            *mem.mut_8(mem.r_i().de.r16()) = mem.r().a;
            cycles += 1;
        }
        "00_100_010" => {
            // LD (HLI) A
            *mem.mut_8(mem.r_i().hl.r16()) = mem.r().a;
            mem.r().hl.uinc16();
            cycles += 1;
        }
        "00_110_010" => {
            // LD (HLD) A
            *mem.mut_8(mem.r_i().hl.r16()) = mem.r().a;
            mem.r().hl.udec16();
            cycles += 1;
        }
        "00_dd0_001" => {
            // LD dd nn
            let val = u8s_to_u16(mem.read_8(mem.r_i().pc + 1), mem.read_8(mem.r_i().pc));
            match d {
                0 => {
                    mem.r().bc.s16(val);
                }
                0b00000001 => {
                    mem.r().de.s16(val);
                }
                0b00000010 => {
                    mem.r().hl.s16(val);
                }
                0b00000011 => mem.r().sp = val,
                _ => panic!("Unknown load code in LD dd nn: {:#b}", d),
            };
            mem.r().pc += 2;
            cycles += 2;
        }
        "00_001_000" => {
            // LD (nn), SP
            let vals = u16_to_u8s(mem.r().sp);
            let addr = u8s_to_u16(mem.read_8(mem.r_i().pc + 1), mem.read_8(mem.r_i().pc));
            mem.write_8(addr, vals.0);
            mem.write_8(addr + 1, vals.1);
            mem.r().pc += 2;
            cycles += 4;
        }
        "00_010_000" => {
            // STOP
            // see page 23
            todo!("Need to figure out interrupts first before STOP");
        }
        "00_110_111" => {
            // SCF
            mem.r().f.set(RegisterFlags::CY, true);
            mem.r().f.set(RegisterFlags::H, false);
            mem.r().f.set(RegisterFlags::N, false);
        }
        "00_111_111" => {
            // CCF
            let cy = mem.r_i().f.contains(RegisterFlags::CY);
            mem.r().f.set(RegisterFlags::CY, !cy);
            mem.r().f.set(RegisterFlags::H, false);
            mem.r().f.set(RegisterFlags::N, false);
        }
        "01_110_110" => {
            // HALT
            // see page 23
            todo!("Need to figure out interrupts first before HALT");
        }
        "01_mmm_lll" => {
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
            mem.r().a = add_8(mem.r().a, get_register_val(mem, l), mem, false);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_001_lll" => {
            // ADC A, r and ADC A, (HL)
            mem.r().a = add_8(mem.r().a, get_register_val(mem, l), mem, true);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_010_lll" => {
            // SUB A, r and SUB A, (HL)
            mem.r().a = sub_8(mem.r().a, get_register_val(mem, l), mem, false);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_011_lll" => {
            // SBC A, r and SBC A, (HL)
            mem.r().a = sub_8(mem.r().a, get_register_val(mem, l), mem, true);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_100_lll" => {
            // AND A, r and AND A, (HL)
            mem.r().a = and_8(mem.r().a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_101_lll" => {
            // XOR A, r and XOR A, (HL)
            mem.r().a = xor_8(mem.r().a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_110_lll" => {
            // OR A, r and OR A, (HL)
            mem.r().a = or_8(mem.r().a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "10_111_lll" => {
            // CP A, r and CP A, (HL)
            cp_8(mem.r().a, get_register_val(mem, l), mem);
            if l == 0b00000110 {
                cycles += 1;
            }
        }
        "11_000_011" => {
            // JP nn
            let addr = u8s_to_u16(mem.read_8(mem.r_i().pc + 1), mem.read_8(mem.r_i().pc));
            mem.r().pc = addr;
            cycles += 3;
        }
        "11_0cc_010" => {
            // JP cc, nn
            if check_jump_condition(c, mem) {
                let addr = u8s_to_u16(mem.read_8(mem.r_i().pc + 1), mem.read_8(mem.r_i().pc));
                mem.r().pc = addr;
                cycles += 3;
            } else {
                mem.r().pc += 2;
                cycles += 2;
            }
        }
        "11_000_110" => {
            // ADD A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = add_8(mem.r().a, n, mem, false);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_001_001" => {
            // RET
            ret(mem);
            cycles += 3;
        }
        "11_001_011" => {
            // CB prefix
            let next_instruction = mem.read_8(mem.r_i().pc);
            mem.r().pc += 1;
            // all CB instructions take at least 2 cycles
            cycles += 1;

            #[bitmatch]
            match next_instruction {
                "00_000_rrr" => {
                    // RLC r, RLC (HL)
                    let result = rlc(get_register_val(mem, r), mem, true);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_001_rrr" => {
                    // RRC r, RRC (HL)
                    let result = rrc(get_register_val(mem, r), mem, true);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_010_rrr" => {
                    // RL r, RL (HL)
                    let result = rl(get_register_val(mem, r), mem, true);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_011_rrr" => {
                    // RR r, RR (HL)
                    let result = rr(get_register_val(mem, r), mem, true);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_100_rrr" => {
                    // SLA r, SLA (HL)
                    let result = sla(get_register_val(mem, r), mem);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_101_rrr" => {
                    // SRA r, SRA (HL)
                    let result = sra(get_register_val(mem, r), mem);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_110_rrr" => {
                    // SWAP r, SWAP (HL)
                    let result = swap(get_register_val(mem, r), mem);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "00_111_rrr" => {
                    // SRL r, SRL (HL)
                    let result = srl(get_register_val(mem, r), mem);
                    *get_register_mut(mem, r) = result;

                    if r == 0b00000110 {
                        cycles += 2;
                    }
                }
                "01_bbb_rrr" => {
                    // BIT b, r, BIT b, (HL)
                    bit(get_register_val(mem, r), b, mem);

                    if r == 0b00000110 {
                        cycles += 1;
                    }
                }
                "10_bbb_rrr" => {
                    // RES b, r, RES b, (HL)
                    let result = res(get_register_val(mem, r), b);
                    *get_register_mut(mem, r) = result;
                    

                    if r == 0b00000110 {
                        cycles += 1;
                    }
                }
                "11_bbb_rrr" => {
                    // SET b, r, SET b, (HL)
                    let result = set(get_register_val(mem, r), b);
                    *get_register_mut(mem, r) = result;
                    

                    if r == 0b00000110 {
                        cycles += 1;
                    }
                }
            }
        }
        "11_001_101" => {
            // CALL nn
            call(mem);
            cycles += 5;
        }
        "11_001_110" => {
            // ADC A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = add_8(mem.r().a, n, mem, true);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_010_110" => {
            // SUB A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = sub_8(mem.r().a, n, mem, false);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_010_110" => {
            // SBC A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = sub_8(mem.r().a, n, mem, true);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_010_110" => {
            // AND A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = and_8(mem.r().a, n, mem);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_0cc_000" => {
            // RET cc
            if check_jump_condition(c, mem) {
                ret(mem);
                cycles += 4;
            } else {
                cycles += 1;
            }
        }
        "11_011_001" => {
            // RETI
            ret(mem);
            *mem.ime() = true;
            cycles += 3;
        }
        "11_0cc_100" => {
            // CALL cc, nn
            if check_jump_condition(c, mem) {
                call(mem);
                cycles += 5;
            } else {
                cycles += 2;
                mem.r().pc += 2;
            }
        }
        "11_101_000" => {
            // ADD SP, e
            let e = mem.read_8(mem.r_i().pc);
            add_sp_e(e, mem);
            mem.r().pc += 1;
            cycles += 3;
        }
        "11_101_01" => {
            // JP HL
            let addr = mem.r().hl.r16();
            mem.r().pc = addr;
            todo!("JP HL: Check if this actually works properly. addr: {:#x}", addr)
        }
        "11_101_110" => {
            // XOR A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = xor_8(mem.r().a, n, mem);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_110_011" => {
            // DI
            *mem.ime() = false;
        }
        "11_110_110" => {
            // OR A, n
            let n = mem.read_8(mem.r_i().pc);
            mem.r().a = or_8(mem.r().a, n, mem);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_111_011" => {
            *mem.ime() = true;
        }
        "11_111_110" => {
            // CP A, n
            let n = mem.read_8(mem.r_i().pc);
            cp_8(mem.r().a, n, mem);
            mem.r().pc += 1;
            cycles += 1;
        }
        "11_110_010" => {
            // LD A (C)
            mem.r().a = mem.read_8(0xFF00 + mem.r_i().bc.ind.1 as u16);
            cycles += 1;
        }
        "11_100_010" => {
            // LD (C) A
            *mem.mut_8(0xFF00 + mem.r_i().bc.ind.1 as u16) = mem.r().a;
            cycles += 1;
        }
        "11_110_000" => {
            // LD A (FF00 + n)
            mem.r().a = mem.read_8(0xFF00 + mem.read_8(mem.r_i().pc) as u16);
            mem.r().pc += 1;
            cycles += 2;
        }
        "11_100_000" => {
            // LD (FF00 + n) A
            *mem.mut_8(0xFF00 + mem.read_8(mem.r_i().pc) as u16) = mem.r().a;
            mem.r().pc += 1;
            cycles += 2;
        }
        "11_111_010" => {
            // LD A nn
            let addr = u8s_to_u16(mem.read_8(mem.r_i().pc), mem.read_8(mem.r_i().pc + 1));
            mem.r().a = mem.read_8(addr);
            mem.r().pc += 2;
            cycles += 3;
            todo!("LD A nn: check order of hi and lo. a: {:#x}", mem.r().a)
        }
        "11_101_010" => {
            // LD nn A
            let addr = u8s_to_u16(mem.read_8(mem.r_i().pc), mem.read_8(mem.r_i().pc + 1));
            *mem.mut_8(addr) = mem.r().a;
            mem.r().pc += 2;
            cycles += 3;
            todo!("LD nn A: check order of hi and lo. addr: {:#x}", addr)
        }
        "11_111_001" => {
            // LD SP HL
            mem.r().sp = mem.r().hl.r16();
            cycles += 1;
        }
        "11_qq0_101" => {
            // PUSH qq
            let vals = match q {
                0 => mem.r().bc.ind,
                0b00000001 => mem.r().de.ind,
                0b00000010 => mem.r().hl.ind,
                0b00000011 => (mem.r().a, mem.r().f.bits()),
                _ => panic!("Unknown register code in PUSH: {}", q),
            };
            mem.write_8(mem.r_i().sp - 1, vals.0);
            mem.write_8(mem.r_i().sp - 2, vals.1);
            mem.r().sp -= 2;
            cycles += 3;
        }
        "11_qq0_001" => {
            // POP qq
            let vals = (mem.read_8(mem.r_i().sp + 1), mem.read_8(mem.r_i().sp));
            if q == 0b00000011 {
                mem.r().a = vals.0;
                mem.r().set_flags_unchecked(vals.1);
            } else {
                let ptrs = match q {
                    0 => &mut mem.r().bc.ind,
                    0b00000001 => &mut mem.r().de.ind,
                    0b00000010 => &mut mem.r().hl.ind,
                    _ => panic!("Unknown register code in POP: {}", q),
                };
                *ptrs = vals;
            }
            mem.r().sp += 2;
            cycles += 2;
        }
        "11_111_000" => {
            // LDHL SP, e
            let e = mem.read_8(mem.r_i().pc) as i8;
            mem.r().pc += 1;
            let result = add_16_mixed(mem.r().sp, e, mem);
            mem.r().hl.s16(result);
            cycles += 2;
        }
        "11_ttt_111" => {
            // RST t
            let vals = u16_to_u8s(mem.r().pc + 1);
            mem.write_8(mem.r_i().sp - 1, vals.0);
            mem.write_8(mem.r_i().sp - 2, vals.1);
            mem.r().sp -= 2;

            mem.r().pc = t as u16 * 0x08;
        }
        "11_01_0011" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_011_011" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_011_101" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_100_011" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_100_100" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_101_011" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_100_100" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_100_101" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_110_101" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_110_100" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        "11_110_101" => {
            panic!("Invalid instruction {:#b}", current_instruction);
        }
        _ => {
            panic!("Unrecognized instruction {:#b}", current_instruction);
        },
    }

    cycles + 1
}

#[cfg(test)]
mod tests {
    use crate::{memory::MemoryController, memory_controllers::basic_memory::BasicMemory};

    use super::process_instruction;

    #[test]
    fn push_pop_same_val() {
        let mut m = BasicMemory::default();
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
        let mut m = BasicMemory::default();

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
        let mut m = BasicMemory::default();
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
