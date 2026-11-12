//! 按源路径语法处理路径，不让宿主操作系统误解 Windows 盘符或 UNC。

use std::io::{self, ErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPath {
    pub absolute: String,
    pub qgis: String,
    pub foreign: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Root {
    Posix,
    Drive(String),
    Unc(String, String),
    Relative,
}

struct Parsed {
    root: Root,
    parts: Vec<String>,
}

fn invalid() -> io::Error {
    io::Error::new(
        ErrorKind::InvalidInput,
        "路径为空、含 URI/OGR 选项或具有不明确的根语义",
    )
}

fn parse(value: &str) -> io::Result<Parsed> {
    if value.is_empty() || value.contains(['\0', '|', '?', '#']) || value.contains("://") {
        return Err(invalid());
    }
    let windows_rooted = value.starts_with('\\') && !value.starts_with("\\\\");
    if windows_rooted {
        return Err(invalid());
    }
    let value = value.replace('\\', "/");
    let mut rest = value.as_str();
    let root = if let Some(unc) = rest.strip_prefix("//") {
        let mut pieces = unc.splitn(3, '/');
        let server = pieces.next().unwrap_or_default();
        let share = pieces.next().unwrap_or_default();
        if [server, share]
            .iter()
            .any(|s| s.is_empty() || *s == "." || *s == ".." || s.contains(':'))
        {
            return Err(invalid());
        }
        rest = pieces.next().unwrap_or_default();
        Root::Unc(server.to_owned(), share.to_owned())
    } else if rest.as_bytes().get(1) == Some(&b':') {
        if !rest.as_bytes()[0].is_ascii_alphabetic() || rest.as_bytes().get(2) != Some(&b'/') {
            return Err(invalid());
        }
        let drive = rest[..1].to_ascii_uppercase();
        rest = &rest[3..];
        Root::Drive(drive)
    } else if let Some(tail) = rest.strip_prefix('/') {
        rest = tail;
        Root::Posix
    } else {
        Root::Relative
    };
    if rest.contains(':') {
        return Err(invalid());
    }
    let mut parsed = Parsed {
        root,
        parts: Vec::new(),
    };
    append(&mut parsed, rest.split('/'))?;
    Ok(parsed)
}

fn append<'a>(path: &mut Parsed, pieces: impl Iterator<Item = &'a str>) -> io::Result<()> {
    for piece in pieces {
        match piece {
            "" | "." => (),
            ".." if path.parts.last().is_some_and(|p| p != "..") => {
                path.parts.pop();
            }
            ".." if path.root == Root::Relative => path.parts.push(piece.to_owned()),
            ".." => return Err(io::Error::new(ErrorKind::InvalidInput, "路径越过根目录")),
            _ => path.parts.push(piece.to_owned()),
        }
    }
    Ok(())
}

fn absolute(path: &Parsed) -> String {
    let prefix = match &path.root {
        Root::Posix => "/".to_owned(),
        Root::Drive(d) => format!("{d}:/"),
        Root::Unc(server, share) => format!("//{server}/{share}/"),
        Root::Relative => String::new(),
    };
    prefix + &path.parts.join("/")
}

/// 相对源路径以源 MXD 所在目录解释，再相对于输出目录改写；不要求文件存在。
/// 两个目录必须是各自语法中的绝对目录，不能传入 MXD/QGS 文件名。
pub fn resolve(
    path: &str,
    source_directory: &str,
    output_directory: &str,
) -> io::Result<ResolvedPath> {
    let mut source = parse(source_directory)?;
    let output = parse(output_directory)?;
    if source.root == Root::Relative || output.root == Root::Relative {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "源目录和输出目录必须为绝对路径",
        ));
    }
    let input = parse(path)?;
    if input.root == Root::Relative {
        append(&mut source, input.parts.iter().map(String::as_str))?;
    } else {
        source = input;
    }
    let absolute = absolute(&source);
    let foreign = if cfg!(windows) {
        source.root == Root::Posix
    } else {
        source.root != Root::Posix
    };
    let same_root = match (&source.root, &output.root) {
        (Root::Drive(a), Root::Drive(b)) => a.eq_ignore_ascii_case(b),
        (Root::Unc(a, b), Root::Unc(c, d)) => {
            a.eq_ignore_ascii_case(c) && b.eq_ignore_ascii_case(d)
        }
        (Root::Posix, Root::Posix) => true,
        _ => false,
    };
    let qgis = if same_root {
        let windows = source.root != Root::Posix;
        let common = source
            .parts
            .iter()
            .zip(&output.parts)
            .take_while(|(a, b)| {
                if windows {
                    a.eq_ignore_ascii_case(b)
                } else {
                    a == b
                }
            })
            .count();
        let mut parts = vec![".."; output.parts.len() - common];
        parts.extend(source.parts[common..].iter().map(String::as_str));
        let relative = parts.join("/");
        if relative.starts_with("../") {
            relative
        } else {
            format!("./{relative}")
        }
    } else {
        absolute.clone()
    };
    Ok(ResolvedPath {
        absolute,
        qgis,
        foreign,
    })
}
