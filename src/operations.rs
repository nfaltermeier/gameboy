use crate::memory::{Memory, RegisterFlags};

pub fn add_8(a: u8, mut b: u8, m: &mut Memory, carry: bool) -> u8 {
    let al = a & 0x0F;
    let mut bl = b & 0x0F;
    if carry {
        // TODO: check for overflow here?
        bl += 1;
        b += 1;
    }

    let (result, overflow) = a.overflowing_add(b);

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, (al + bl) > 0x0F);
    m.r.f.set(RegisterFlags::N, false);
    m.r.f.set(RegisterFlags::CY, overflow);

    result
}

pub fn add_16_mixed(a: u16, b: i8, m: &mut Memory) -> u16 {
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

    m.r.f.set(RegisterFlags::Z, false);
    m.r.f.set(RegisterFlags::H, half_overflow);
    m.r.f.set(RegisterFlags::N, false);
    m.r.f.set(RegisterFlags::CY, overflow);

    result
}

pub fn sub_8(a: u8, mut b: u8, m: &mut Memory, carry: bool) -> u8 {
    let al = a & 0x0F;
    let mut bl = b & 0x0F;
    if carry {
        // TODO: check for overflow here?
        bl += 1;
        b += 1;
    }

    let (result, overflow) = a.overflowing_sub(b);

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, bl > al);
    m.r.f.set(RegisterFlags::N, true);
    m.r.f.set(RegisterFlags::CY, overflow);

    result
}

pub fn and_8(a: u8, b: u8, m: &mut Memory) -> u8 {
    let result = a & b;

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, false);
    m.r.f.set(RegisterFlags::N, false);
    m.r.f.set(RegisterFlags::CY, false);

    result
}

pub fn or_8(a: u8, b: u8, m: &mut Memory) -> u8 {
    let result = a | b;

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, false);
    m.r.f.set(RegisterFlags::N, false);
    m.r.f.set(RegisterFlags::CY, false);

    result
}

pub fn xor_8(a: u8, b: u8, m: &mut Memory) -> u8 {
    let result = a ^ b;

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, false);
    m.r.f.set(RegisterFlags::N, false);
    m.r.f.set(RegisterFlags::CY, false);

    result
}

pub fn cp_8(a: u8, b: u8, m: &mut Memory) {
    m.r.f.set(RegisterFlags::Z, a == b);
    m.r.f.set(RegisterFlags::H, a > b);
    m.r.f.set(RegisterFlags::N, true);
    m.r.f.set(RegisterFlags::CY, a < b);
}

pub fn inc_8(a: u8, m: &mut Memory) -> u8 {
    let al = a & 0x0F;

    let result = a.wrapping_add(1);

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, (al + 1) > 0x0F);
    m.r.f.set(RegisterFlags::N, false);

    result
}
pub fn inc_82(a: &mut u8, m: &mut Memory) {
    let al = *a & 0x0F;

    *a = a.wrapping_add(1);

    m.r.f.set(RegisterFlags::Z, *a == 0);
    m.r.f.set(RegisterFlags::H, (al + 1) > 0x0F);
    m.r.f.set(RegisterFlags::N, false);
}

pub fn dec_8(a: u8, m: &mut Memory) -> u8 {
    let result = a.wrapping_sub(1);

    m.r.f.set(RegisterFlags::Z, result == 0);
    m.r.f.set(RegisterFlags::H, a == 0);
    m.r.f.set(RegisterFlags::N, false);

    result
}
