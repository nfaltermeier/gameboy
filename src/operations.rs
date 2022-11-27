use crate::memory::{Memory, RegisterFlags};

pub fn add_8(a: u8, b: u8, m: &mut Memory) -> u8 {
    let al = a & 0x0F;
    let bl = b & 0x0F;
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
