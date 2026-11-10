use std::{
    env, io,
    io::{Read, Write},
    path::Path,
};

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
            "usage: mxd2qgz <inspect|strings> <file.mxd> | extract <file.mxd> <stream> <output>",
        ));
    }
    let path = Path::new(path.as_ref().unwrap());
    let mut document = cfb::open(path)?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    if command.as_deref() == Some(std::ffi::OsStr::new("strings")) {
        let mut bytes = Vec::new();
        document.open_stream("/Maps")?.read_to_end(&mut bytes)?;
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
    use super::utf16_candidates;

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
