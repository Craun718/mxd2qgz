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
    if !matches!(
        command.as_deref().and_then(|s| s.to_str()),
        Some("inspect" | "strings")
    ) || path.is_none()
        || args.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: mxd2qgz <inspect|strings> <file.mxd> | extract <file.mxd> <stream> <output> | hex <file.mxd> <stream> <offset> <length>",
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
        for offset in (alignment..bytes.len().saturating_sub(1)).step_by(2) {
            let unit = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            let printable = char::from_u32(unit as u32)
                .is_some_and(|ch| !ch.is_control() && !ch.is_whitespace() || ch == ' ');
            if printable {
                if units.is_empty() {
                    start = offset;
                }
                units.push(unit);
            } else {
                if units.len() >= 4 {
                    result.push((start, String::from_utf16_lossy(&units)));
                }
                units.clear();
            }
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
    use super::{parse_number, utf16_candidates};

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
}
