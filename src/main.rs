use mxd2qgz::mxd_version;

mod input;

use std::{env, io, io::Write, path::Path};

fn main() {
    if let Err(error) = run() {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return;
        }
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let command = args.next();
    let path = args.next();
    if command.as_deref() == Some(std::ffi::OsStr::new("compare")) {
        let other = args.next();
        let stream = args.next();
        if let (Some(path), Some(other), Some(stream), None) = (path, other, stream, args.next()) {
            let mut first = cfb::open(path)?;
            let mut second = cfb::open(other)?;
            let left = input::read_stream(&mut first, Path::new(&stream))?;
            let right = input::read_stream(&mut second, Path::new(&stream))?;
            let mut output = io::BufWriter::new(io::stdout().lock());
            writeln!(
                output,
                "Stream {}: {} vs {} bytes",
                Path::new(&stream).display(),
                left.len(),
                right.len()
            )?;
            for (start, end) in differing_ranges(&left, &right) {
                writeln!(output, "0x{start:08x}..0x{end:08x}")?;
            }
            return output.flush();
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz compare <first.mxd> <second.mxd> <stream>",
        ));
    }
    if command.as_deref() == Some(std::ffi::OsStr::new("hex")) {
        let stream = args.next();
        let offset = args.next();
        let length = args.next();
        if let (Some(path), Some(stream), Some(offset), Some(length), None) =
            (path, stream, offset, length, args.next())
        {
            let offset = parse_number(&offset)?;
            let length = parse_number(&length)?;
            let mut document = cfb::open(path)?;
            let bytes = input::read_stream(&mut document, Path::new(&stream))?;
            let selected = input::range(&bytes, offset, length)?;
            let mut output = io::BufWriter::new(io::stdout().lock());
            for (line, chunk) in selected.chunks(16).enumerate() {
                write!(output, "0x{:08x}  ", offset + line * 16)?;
                for i in 0..16 {
                    if let Some(byte) = chunk.get(i) {
                        write!(output, "{byte:02x} ")?;
                    } else {
                        write!(output, "   ")?;
                    }
                }
                write!(output, " |")?;
                for &byte in chunk {
                    write!(
                        output,
                        "{}",
                        if byte.is_ascii_graphic() || byte == b' ' {
                            byte as char
                        } else {
                            '.'
                        }
                    )?;
                }
                writeln!(output, "|")?;
            }
            return output.flush();
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz hex <file.mxd> <stream> <offset> <length>",
        ));
    }
    if command.as_deref() == Some(std::ffi::OsStr::new("extract")) {
        let stream = args.next();
        let output = args.next();
        if let (Some(path), Some(stream), Some(output), None) = (path, stream, output, args.next())
        {
            let mut document = cfb::open(path)?;
            let mut source = document.open_stream(Path::new(&stream))?;
            let mut destination = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            io::copy(&mut source, &mut destination)?;
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz extract <file.mxd> <stream> <output>",
        ));
    }
    if command.as_deref() == Some(std::ffi::OsStr::new("version")) {
        if let (Some(path), None) = (path, args.next()) {
            let mut document = cfb::open(&path)?;
            let bytes = input::read_stream(&mut document, "/Version").map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!(
                        "无法读取 {} 的 /Version 数据流：{error}",
                        Path::new(&path).display()
                    ),
                )
            })?;
            let report = mxd_version::identify_version(&bytes)?;
            let mut output = io::BufWriter::new(io::stdout().lock());
            writeln!(output, "Stream /Version: {} bytes", bytes.len())?;
            writeln!(output, "Branch: {}", report.branch.description())?;
            writeln!(output, "Text: {:?}", report.text)?;
            writeln!(
                output,
                "Compatibility short: {}",
                report.compatibility_short
            )?;
            match report.object_count {
                Some(count) => writeln!(output, "Object count: {count}")?,
                None => writeln!(output, "Object count: none")?,
            }
            writeln!(output, "Remaining bytes: {}", report.remaining_bytes)?;
            writeln!(output, "Note: {}", report.note)?;
            return output.flush();
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz version <file.mxd>",
        ));
    }
    if !matches!(
        command.as_deref().and_then(|s| s.to_str()),
        Some("inspect" | "strings")
    ) || path.is_none()
        || args.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz <inspect|strings> <file.mxd> | version <file.mxd> | compare <first.mxd> <second.mxd> <stream> | extract <file.mxd> <stream> <output> | hex <file.mxd> <stream> <offset> <length>",
        ));
    }
    let path = Path::new(path.as_ref().unwrap());
    let mut document = cfb::open(path)?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    if command.as_deref() == Some(std::ffi::OsStr::new("strings")) {
        let bytes = input::read_stream(&mut document, "/Maps")?;
        writeln!(
            output,
            "UTF-16LE candidates in /Maps (offsets are relative to the stream):"
        )?;
        for (offset, value) in utf16_candidates(&bytes) {
            writeln!(output, "0x{offset:08x}  {value}")?;
        }
        return output.flush();
    }
    writeln!(output, "Document: {}", path.display())?;
    for entry in document.walk().filter(|entry| entry.is_stream()) {
        writeln!(output, "{:>10}  {}", entry.len(), entry.path().display())?;
    }
    output.flush()
}

// Compare equal stream offsets, including any unmatched tail. Ranges are half-open.
fn differing_ranges(left: &[u8], right: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = None;
    for offset in 0..left.len().max(right.len()) {
        if left.get(offset) != right.get(offset) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            ranges.push((begin, offset));
        }
    }
    if let Some(begin) = start {
        ranges.push((begin, left.len().max(right.len())));
    }
    ranges
}

fn parse_number(value: &std::ffi::OsStr) -> io::Result<usize> {
    let value = value
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid number"))?;
    let parsed = if let Some(hex) = value.strip_prefix("0x") {
        usize::from_str_radix(hex, 16)
    } else {
        value.parse()
    };
    parsed.map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid number"))
}

fn utf16_candidates(bytes: &[u8]) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    for alignment in 0..2 {
        let mut start = alignment;
        let mut units = Vec::new();
        let mut offset = alignment;
        while offset < bytes.len().saturating_sub(1) {
            let unit = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            // Decode a surrogate pair as one scalar while retaining byte offsets.
            // An unpaired surrogate ends a candidate instead of inserting U+FFFD.
            let pair = if (0xd800..=0xdbff).contains(&unit) {
                bytes.get(offset + 2..).and_then(|tail| {
                    let low = u16::from_le_bytes(tail.get(..2)?.try_into().ok()?);
                    (0xdc00..=0xdfff).contains(&low).then_some(low)
                })
            } else {
                None
            };
            let scalar = pair.map_or(unit as u32, |low| {
                0x10000 + (((unit as u32 - 0xd800) << 10) | (low as u32 - 0xdc00))
            });
            let printable = char::from_u32(scalar)
                .is_some_and(|ch| (!ch.is_control() && !ch.is_whitespace()) || ch == ' ');
            if printable {
                if units.is_empty() {
                    start = offset;
                }
                units.push(unit);
                if let Some(low) = pair {
                    units.push(low);
                }
            } else {
                if units.len() >= 4 {
                    result.push((start, String::from_utf16_lossy(&units)));
                }
                units.clear();
            }
            offset += if pair.is_some() { 4 } else { 2 };
        }
        if units.len() >= 4 {
            result.push((start, String::from_utf16_lossy(&units)));
        }
    }
    result.sort_by_key(|(offset, _)| *offset);
    result
}

#[cfg(test)]
mod tests {
    use super::{differing_ranges, parse_number, utf16_candidates};

    #[test]
    fn compares_equal_offsets_and_unmatched_tails() {
        assert_eq!(differing_ranges(b"", b""), vec![]);
        assert_eq!(differing_ranges(b"abcd", b"abcd"), vec![]);
        assert_eq!(differing_ranges(b"abcd", b"axcyz"), vec![(1, 2), (3, 5)]);
        assert_eq!(differing_ranges(b"abc", b"a"), vec![(1, 3)]);
    }

    #[test]
    fn parses_decimal_and_hex_offsets() {
        assert_eq!(parse_number("0x10".as_ref()).unwrap(), 16);
        assert_eq!(parse_number("16".as_ref()).unwrap(), 16);
        assert!(parse_number("-1".as_ref()).is_err());
    }

    #[test]
    fn extracts_unicode_on_both_alignments_with_stream_offsets() {
        let mut data = vec![0];
        for c in "矢量2.gdb".encode_utf16() {
            data.extend_from_slice(&c.to_le_bytes());
        }
        data.extend_from_slice(&[0, 0]);
        assert!(utf16_candidates(&data).contains(&(1, "矢量2.gdb".into())));
    }

    #[test]
    fn candidates_preserve_surrogate_pairs_on_both_alignments() {
        for alignment in 0..2 {
            let mut data = vec![0; alignment];
            data.extend("地图😀图层".encode_utf16().flat_map(u16::to_le_bytes));
            data.extend_from_slice(&[0, 0, 0xff]);
            assert!(utf16_candidates(&data).contains(&(alignment, "地图😀图层".into())));
        }
    }

    #[test]
    fn malformed_surrogates_split_candidates_without_replacement() {
        for invalid in [vec![0xd800], vec![0xdc00], vec![0xd800, 0xd800]] {
            let units: Vec<u16> = "abcd"
                .encode_utf16()
                .chain(invalid)
                .chain("efgh".encode_utf16())
                .chain([0])
                .collect();
            let bytes: Vec<_> = units.iter().flat_map(|unit| unit.to_le_bytes()).collect();
            let candidates = utf16_candidates(&bytes);
            assert!(candidates.contains(&(0, "abcd".into())));
            assert!(candidates.contains(&((units.len() - 5) * 2, "efgh".into())));
            assert!(
                candidates
                    .iter()
                    .all(|(_, value)| !value.contains('\u{fffd}'))
            );
        }
        let mut bytes: Vec<_> = "abcd".encode_utf16().flat_map(u16::to_le_bytes).collect();
        bytes.extend_from_slice(&[0x00, 0xd8, 0xff]);
        assert!(utf16_candidates(&bytes).contains(&(0, "abcd".into())));
        assert!(utf16_candidates(&[]).is_empty());
        assert!(utf16_candidates(&[0xff]).is_empty());
    }
}
