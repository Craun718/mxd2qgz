use std::io::{self, Read, Seek};
use std::path::Path;

/// Read a whole OLE stream without interpreting its contents.
pub fn read_stream<F: Read + Seek>(
    document: &mut cfb::CompoundFile<F>,
    path: impl AsRef<Path>,
) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    document.open_stream(path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Select a stream-relative range, rejecting truncation and arithmetic overflow.
pub fn range(bytes: &[u8], offset: usize, length: usize) -> io::Result<&[u8]> {
    let end = offset
        .checked_add(length)
        .filter(|&end| end <= bytes.len())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "range exceeds stream"))?;
    Ok(&bytes[offset..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    #[test]
    fn reads_streams_and_reports_missing_streams() {
        let mut document = cfb::CompoundFile::create(Cursor::new(Vec::new())).unwrap();
        document
            .create_stream("/Maps")
            .unwrap()
            .write_all(b"abc")
            .unwrap();
        assert_eq!(read_stream(&mut document, "/Maps").unwrap(), b"abc");
        assert!(read_stream(&mut document, "/Missing").is_err());
    }

    #[test]
    fn checks_ranges_including_empty_and_overflow() {
        let bytes = b"abcd";
        assert_eq!(range(bytes, 1, 2).unwrap(), b"bc");
        assert_eq!(range(bytes, 4, 0).unwrap(), b"");
        for (offset, length) in [(3, 2), (5, 0), (usize::MAX, 2)] {
            assert_eq!(
                range(bytes, offset, length).unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
        }
    }
}
