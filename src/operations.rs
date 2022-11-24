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
