use crate::{
    memory::{MemoryController, RegisterFlags},
    opcodes::{u16_to_u8s, u8s_to_u16},
};

pub fn add_8(a: u8, mut b: u8, m: &mut dyn MemoryController, carry: bool) -> u8 {
    let al = a & 0x0F;
    let mut bl = b & 0x0F;
    if carry {
        // TODO: check for overflow here?
        bl += 1;
        b += 1;
    }

    let (result, overflow) = a.overflowing_add(b);

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, (al + bl) > 0x0F);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, overflow);

    result
}

pub fn add_16(a: u16, b: u16, m: &mut dyn MemoryController) -> u16 {
    let al = a & 0x0FFF;
    let bl = b & 0x0FFF;

    let (result, overflow) = a.overflowing_add(b);

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, (al + bl) > 0x0FFF);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, overflow);

    result
}

pub fn add_16_mixed(a: u16, b: i8, m: &mut dyn MemoryController) -> u16 {
    let result: u16;
    let overflow: bool;
    let half_overflow: bool;
    // TODO: figure out how to set carry flags when underflowing
    if b >= 0 {
        (result, overflow) = a.overflowing_add(b as u16);
        half_overflow = (result & 0xF000) != (a & 0xF000);
    } else {
        result = a.wrapping_sub((-b) as u16);
        overflow = false;
        half_overflow = false;
    }

    m.r().f.set(RegisterFlags::Z, false);
    m.r().f.set(RegisterFlags::H, half_overflow);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, overflow);

    result
}

pub fn sub_8(a: u8, mut b: u8, m: &mut dyn MemoryController, carry: bool) -> u8 {
    let al = a & 0x0F;
    let mut bl = b & 0x0F;
    if carry {
        // TODO: check for overflow here?
        bl += 1;
        b += 1;
    }

    let (result, overflow) = a.overflowing_sub(b);

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, bl > al);
    m.r().f.set(RegisterFlags::N, true);
    m.r().f.set(RegisterFlags::CY, overflow);

    result
}

pub fn and_8(a: u8, b: u8, m: &mut dyn MemoryController) -> u8 {
    let result = a & b;

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, false);

    result
}

pub fn or_8(a: u8, b: u8, m: &mut dyn MemoryController) -> u8 {
    let result = a | b;

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, false);

    result
}

pub fn xor_8(a: u8, b: u8, m: &mut dyn MemoryController) -> u8 {
    let result = a ^ b;

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, false);

    result
}

pub fn cp_8(a: u8, b: u8, m: &mut dyn MemoryController) {
    m.r().f.set(RegisterFlags::Z, a == b);
    m.r().f.set(RegisterFlags::H, a > b);
    m.r().f.set(RegisterFlags::N, true);
    m.r().f.set(RegisterFlags::CY, a < b);
}

pub fn inc_8(a: u8, m: &mut dyn MemoryController) -> u8 {
    let al = a & 0x0F;

    let result = a.wrapping_add(1);

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, (al + 1) > 0x0F);
    m.r().f.set(RegisterFlags::N, false);

    result
}

pub fn inc_16(a: u16) -> u16 {
    // todo: wrapping behavior?
    a.wrapping_add(1)
}

pub fn dec_16(a: u16) -> u16 {
    // todo: wrapping behavior?
    a.wrapping_sub(1)
}

pub fn dec_8(a: u8, m: &mut dyn MemoryController) -> u8 {
    let result = a.wrapping_sub(1);

    m.r().f.set(RegisterFlags::Z, result == 0);
    m.r().f.set(RegisterFlags::H, a == 0);
    m.r().f.set(RegisterFlags::N, false);

    result
}

pub fn add_sp_e(e: u8, m: &mut dyn MemoryController) {
    m.r().sp = m.r().sp.wrapping_add(e.into());

    m.r().f.set(RegisterFlags::Z, false);
    m.r().f.set(RegisterFlags::N, false);
}

pub fn rlc(a: u8, m: &mut dyn MemoryController, rlca: bool) -> u8 {
    if rlca {
        m.r().f.set(RegisterFlags::Z, false);
    } else {
        m.r().f.set(RegisterFlags::Z, a == 0);
    }
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 0x80) != 0);

    a.rotate_left(1)
}

pub fn rrc(a: u8, m: &mut dyn MemoryController, rrca: bool) -> u8 {
    if rrca {
        m.r().f.set(RegisterFlags::Z, false);
    } else {
        m.r().f.set(RegisterFlags::Z, a == 0);
    }
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 1) != 0);

    a.rotate_right(1)
}

pub fn rl(a: u8, m: &mut dyn MemoryController, rla: bool) -> u8 {
    let mut result = a << 1;
    if m.r().f.intersects(RegisterFlags::CY) {
        result |= 1;
    }

    if rla {
        m.r().f.set(RegisterFlags::Z, false);
    } else {
        m.r().f.set(RegisterFlags::Z, result == 0);
    }
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 0x80) != 0);

    result
}

pub fn rr(a: u8, m: &mut dyn MemoryController, rra: bool) -> u8 {
    let mut result = a >> 1;
    if m.r().f.intersects(RegisterFlags::CY) {
        result |= 0x80;
    }

    if rra {
        m.r().f.set(RegisterFlags::Z, false);
    } else {
        m.r().f.set(RegisterFlags::Z, result == 0);
    }
    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 1) != 0);

    result
}

pub fn sla(a: u8, m: &mut dyn MemoryController) -> u8 {
    let result = a << 1;

    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 0b10000000) != 0);
    m.r().f.set(RegisterFlags::Z, result == 0);

    result
}

pub fn sra(a: u8, m: &mut dyn MemoryController) -> u8 {
    // Cast as i8 to use arithmetic shift instead of logical shift
    let result = a as i8 >> 1;

    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 0b00000001) != 0);
    m.r().f.set(RegisterFlags::Z, result == 0);

    result as u8
}

pub fn srl(a: u8, m: &mut dyn MemoryController) -> u8 {
    let result = a >> 1;

    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, (a & 0b00000001) != 0);
    m.r().f.set(RegisterFlags::Z, result == 0);

    result
}

pub fn swap(a: u8, m: &mut dyn MemoryController) -> u8 {
    let lower = a & 0x0F;
    let upper = a & 0xF0;
    let result = (lower << 4) + (upper >> 4);

    m.r().f.set(RegisterFlags::H, false);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::CY, false);
    m.r().f.set(RegisterFlags::Z, result == 0);

    result
}

// swap order of parameters to match notation?
pub fn bit(a: u8, b: u8, m: &mut dyn MemoryController) {
    let target_bit = a & (1 << b);

    m.r().f.set(RegisterFlags::H, true);
    m.r().f.set(RegisterFlags::N, false);
    m.r().f.set(RegisterFlags::Z, target_bit == 0);
}

// swap order of parameters to match notation?
pub fn set(a: u8, b: u8) -> u8 {
    todo!("Check if the 0 behavior was a mistake in the programming manual");
    if b == 0 {
        a
    } else {
        a | (1 << (b - 1))
    }
}

// swap order of parameters to match notation?
pub fn res(a: u8, b: u8) -> u8 {
    a & !(1 << b)
}
