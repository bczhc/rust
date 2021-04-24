#[derive(Debug, Eq, PartialEq)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

pub fn get_endianness() -> Endianness {
    let a: i16 = 0x1234;
    let p = &a as *const i16 as *const i8;
    unsafe {
        return if *p == 0x12_i8 {
            Endianness::BigEndian
        } else {
            Endianness::LittleEndian
        };
    }
}
