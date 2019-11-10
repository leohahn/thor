use std::io::Write;

pub(crate) fn write_integer(buf: &mut Vec<u8>, int: i64) {
    write!(buf, "i{}e", int).unwrap();
}

pub(crate) fn write_unsigned(buf: &mut Vec<u8>, int: u64) {
    write!(buf, "i{}e", int).unwrap();
}
