//! Bounds-checked primitives for investigating binary streams.
//!
//! These operations do not imply any MXD field encoding or object structure.

use std::io::{self, ErrorKind};

#[derive(Debug, Clone)]
pub struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn position(&self) -> usize {
        self.offset
    }

    /// Read exactly `length` bytes. Failure leaves the position unchanged.
    pub fn read_bytes(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|&end| end <= self.bytes.len())
            .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "read exceeds stream"))?;
        let result = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(result)
    }

    pub fn read_u32_le(&mut self) -> io::Result<u32> {
        let bytes: [u8; 4] = self.read_bytes(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_values_and_empty_tail() {
        let mut reader = Reader::new(&[0x78, 0x56, 0x34, 0x12, 9]);
        assert_eq!(reader.read_u32_le().unwrap(), 0x12345678);
        assert_eq!(reader.read_bytes(1).unwrap(), &[9]);
        assert_eq!(reader.read_bytes(0).unwrap(), &[]);
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn rejects_truncation_and_overflow_without_advancing() {
        let mut reader = Reader::new(&[1, 2, 3]);
        reader.read_bytes(1).unwrap();
        for length in [3, usize::MAX] {
            assert_eq!(
                reader.read_bytes(length).unwrap_err().kind(),
                ErrorKind::UnexpectedEof
            );
            assert_eq!(reader.position(), 1);
        }
        assert_eq!(
            reader.read_u32_le().unwrap_err().kind(),
            ErrorKind::UnexpectedEof
        );
        assert_eq!(reader.position(), 1);
    }
}
