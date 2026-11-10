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

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    /// A child cannot read outside its declared range. Positions are child-relative.
    pub fn read_region(&mut self, length: usize) -> io::Result<Reader<'a>> {
        Ok(Reader::new(self.read_bytes(length)?))
    }

    pub fn read_u16_le(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }

    pub fn read_f64_le(&mut self) -> io::Result<f64> {
        Ok(f64::from_le_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }

    /// Decode exactly the supplied byte length, without interpreting a prefix
    /// or stripping terminators. All failures leave the cursor unchanged.
    pub fn read_utf16_le(&mut self, byte_length: usize) -> io::Result<String> {
        let start = self.offset;
        if !byte_length.is_multiple_of(2) {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("odd UTF-16LE byte length at offset 0x{start:x}"),
            ));
        }
        let mut trial = self.clone();
        let bytes = trial.read_bytes(byte_length)?;
        let units: Vec<_> = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        let value = String::from_utf16(&units).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid UTF-16LE at offset 0x{start:x}"),
            )
        })?;
        self.offset = trial.offset;
        Ok(value)
    }

    /// Read exactly `length` bytes. Failure leaves the position unchanged.
    pub fn read_bytes(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|&end| end <= self.bytes.len())
            .ok_or_else(|| {
                io::Error::new(
                    ErrorKind::UnexpectedEof,
                    format!(
                        "read of {length} bytes at offset 0x{:x} exceeds stream",
                        self.offset
                    ),
                )
            })?;
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
    fn decodes_exact_unicode_bytes_and_keeps_terminators() {
        let bytes: Vec<_> = "地图😀\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut reader = Reader::new(&bytes);
        assert_eq!(reader.read_utf16_le(0).unwrap(), "");
        assert_eq!(reader.read_utf16_le(bytes.len()).unwrap(), "地图😀\0");
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn failed_string_reads_are_transactional() {
        for bytes in [&[0x00, 0xd8][..], &[0x00, 0xdc][..]] {
            let mut reader = Reader::new(bytes);
            assert!(reader.read_utf16_le(2).is_err());
            assert_eq!(reader.position(), 0);
        }
        let mut reader = Reader::new(&[0, 0]);
        for length in [1, 4, usize::MAX] {
            assert!(reader.read_utf16_le(length).is_err());
            assert_eq!(reader.position(), 0);
        }
    }

    #[test]
    fn regions_enforce_boundaries_and_parent_progress() {
        let mut reader = Reader::new(&[1, 0, 2, 0]);
        let mut child = reader.read_region(2).unwrap();
        assert_eq!(child.read_u16_le().unwrap(), 1);
        assert!(child.read_bytes(1).is_err());
        assert_eq!(reader.position(), 2);
        assert!(reader.read_region(3).is_err());
        assert_eq!(reader.read_u16_le().unwrap(), 2);
        assert_eq!(reader.read_region(0).unwrap().remaining(), 0);
        let bytes = 12.5_f64.to_le_bytes();
        assert_eq!(Reader::new(&bytes).read_f64_le().unwrap(), 12.5);
        assert!(Reader::new(&bytes[..7]).read_f64_le().is_err());
    }

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
